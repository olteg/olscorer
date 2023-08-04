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

use crate::audio_utils::AudioData;
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
        // Split samples into frames
        let frame_width = 4096;
        let step_size = 1024;

        let frames = audio_data.get_frames(frame_width, step_size, None, None);

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
}
