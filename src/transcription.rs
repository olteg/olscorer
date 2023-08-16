/*
 * Olscorer
 * Automatic Music Transcription Software
 *
 * Copyright (C) 2023  Oleg Tretieu
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::audio_utils::{root_mean_square, AudioData, Frame};
use crate::pitch_detection::{Mpm, PitchDetector};
use std::fmt;

/// Struct representing a musical note
pub struct Note {
    /// Name of the note, including its pitch and octave
    pub name: NoteName,

    /// Sample at which the note starts playing
    pub start: usize,

    /// Duration of the note (in samples)
    pub duration: usize,
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Note: {}, Start: {}, Duration: {}",
            self.name, self.start, self.duration
        )
    }
}

/// Enum representing the name of a musical note, consisting of the note's
/// letter name and octave
#[derive(PartialEq)]
pub enum NoteName {
    A(u8),
    ASharp(u8),
    B(u8),
    C(u8),
    CSharp(u8),
    D(u8),
    DSharp(u8),
    E(u8),
    F(u8),
    FSharp(u8),
    G(u8),
    GSharp(u8),
}

impl NoteName {
    /// Returns the note name that most closely corresponds to the
    /// given pitch
    fn from_pitch(pitch: f64) -> NoteName {
        let note_num = (12.0 * (pitch / 440.0).log(2.0) + 48.5).floor() as i32;
        let mut note_index = note_num % 12;
        if note_index < 0 {
            note_index = 12 + note_index;
        }
        // Add 9 to the note number to account for the fact that the octave
        // changes at C notes.
        // For example, the note with note number 3 (C1) should be
        // in octave 1 instead of octave 0
        let octave = ((note_num + 9) as f64 / 12.0).floor() as u8;

        match note_index {
            0 => NoteName::A(octave),
            1 => NoteName::ASharp(octave),
            2 => NoteName::B(octave),
            3 => NoteName::C(octave),
            4 => NoteName::CSharp(octave),
            5 => NoteName::D(octave),
            6 => NoteName::DSharp(octave),
            7 => NoteName::E(octave),
            8 => NoteName::F(octave),
            9 => NoteName::FSharp(octave),
            10 => NoteName::G(octave),
            11 => NoteName::GSharp(octave),
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for NoteName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            NoteName::A(octave) => write!(f, "A{}", octave),
            NoteName::ASharp(octave) => write!(f, "A#{}", octave),
            NoteName::B(octave) => write!(f, "B{}", octave),
            NoteName::C(octave) => write!(f, "C{}", octave),
            NoteName::CSharp(octave) => write!(f, "C#{}", octave),
            NoteName::D(octave) => write!(f, "D{}", octave),
            NoteName::DSharp(octave) => write!(f, "D#{}", octave),
            NoteName::E(octave) => write!(f, "E{}", octave),
            NoteName::F(octave) => write!(f, "F{}", octave),
            NoteName::FSharp(octave) => write!(f, "F#{}", octave),
            NoteName::G(octave) => write!(f, "G{}", octave),
            NoteName::GSharp(octave) => write!(f, "G#{}", octave),
        }
    }
}

struct PitchFrame {
    start_pos: usize,
    frame_width: usize,
    pitch: Option<f64>,
}

/// Struct for music transcription functionalities
pub struct Transcriber;

impl Transcriber {
    /// Finds the musical notes in the audio data
    ///
    /// Returns a vector of the notes detected in the audio.
    pub fn get_notes(audio_data: AudioData) -> Vec<Note> {
        // Scale samples
        let abs_max_value = match audio_data
            .samples
            .iter()
            .max_by(|a, b| a.abs().total_cmp(&b.abs()))
        {
            Some(value) => *value,
            None => return vec![],
        }
        .abs();

        let samples: Vec<f64> = audio_data
            .samples
            .iter()
            .map(|x| {
                if abs_max_value >= f64::EPSILON {
                    x / abs_max_value
                } else {
                    0.0
                }
            })
            .collect();

        let audio_data = AudioData {
            sample_rate: audio_data.sample_rate,
            duration: audio_data.duration,
            samples: samples.clone(),
        };

        let onsets = Transcriber::get_onsets(&audio_data);

        let frames = audio_data
            .get_frames_by_index(onsets)
            .expect("Error getting frames");

        // Limit frame width to 8192 samples
        let frames = frames.iter().map(|f| {
            let frame_width = if f.samples.len() < 8192 {
                f.samples.len()
            } else {
                8192
            };
            let samples: Vec<f64> = f.samples[0..frame_width].to_vec();
            Frame {
                start_pos: f.start_pos,
                samples: samples,
            }
        });

        // Filter out silent frames by removing frames where the RMS is less
        // than 20% of the RMS of the entire audio
        // TODO: Implement a more sophisticated algorithm for filtering out
        // silent frames
        let audio_rms =
            root_mean_square(samples.into_iter().map(|s| s as f64).collect()).unwrap_or(0.0);

        let frames: Vec<Frame> = frames
            .into_iter()
            .filter(|f| root_mean_square(f.samples.clone()).unwrap_or(0.0) >= 0.2 * audio_rms)
            .collect();

        // Get the pitch in each frame
        let mpm = Mpm::new(0.7, audio_data.sample_rate);
        let pitches = frames
            .into_iter()
            .map(|frame| PitchFrame {
                start_pos: frame.start_pos,
                frame_width: frame.samples.len(),
                pitch: mpm
                    .clone()
                    .get_pitch(frame.samples.into_iter().map(|x| x as f64).collect()),
            })
            .collect::<Vec<PitchFrame>>();

        // Exclude frames where no pitch was detected
        let pitch_frames: Vec<PitchFrame> =
            pitches.into_iter().filter(|p| p.pitch.is_some()).collect();

        let notes: Vec<Note> = pitch_frames
            .iter()
            .map(|pitch_frame| Note {
                name: NoteName::from_pitch(pitch_frame.pitch.unwrap()),
                start: pitch_frame.start_pos,
                duration: pitch_frame.frame_width,
            })
            .collect();

        notes
    }

    /// Finds the onsets of notes in the audio
    ///
    /// Returns a vector of indices at which note onsets were detected.
    fn get_onsets(audio_data: &AudioData) -> Vec<usize> {
        // Calculate envelope
        let onset_frame_width = 1600;
        let onset_frames = audio_data.get_frames(onset_frame_width, onset_frame_width, None, None);

        // Make all samples positive
        let abs_frames: Vec<Vec<f64>> = onset_frames
            .iter()
            .map(|f| f.samples.iter().map(|x| x.abs()).collect::<Vec<f64>>())
            .collect();

        let envelope: Vec<f64> = abs_frames
            .into_iter()
            .map(|samples| {
                samples
                    .into_iter()
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap_or(0.0)
            })
            .collect();

        let mut indices = vec![];

        // The index of each envelope value should be at the centre of its corresponding frame
        for frame in onset_frames {
            indices.push(frame.start_pos + onset_frame_width / 2);
        }

        // Get differences between consecutive elements of the envelope
        let mut differences = vec![0.0];

        for i in 1..envelope.len() {
            differences.push(envelope[i] - envelope[i - 1]);
        }

        // Get the onsets
        let mut onsets = vec![];
        let mut add_onset = true;
        let difference_threshold = 0.125;

        for i in 0..indices.len() {
            if i < differences.len()
                && differences[(indices[i] - onset_frame_width / 2) / onset_frame_width]
                    > difference_threshold
                && add_onset
            {
                onsets.push(onset_frame_width * i + onset_frame_width / 2);
                add_onset = false;
            } else {
                add_onset = true;
            }
        }

        onsets
    }
}
