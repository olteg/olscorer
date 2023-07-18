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

use crate::errors::WavFileError::{UnsupportedBitDepth, UnsupportedChannelCount};
use hound::{SampleFormat, WavReader};
use std::error::Error;

const MAX_24BIT: i32 = 16777215;

pub struct WavFileData {
    /// Sample rate (in Hz)
    pub sample_rate: u32,
    /// Duration (in samples) of the wav file
    pub duration: u32,
    pub samples: Vec<f32>,
}

pub fn read_wav_file(filepath: std::path::PathBuf) -> Result<WavFileData, Box<dyn Error>> {
    let reader = WavReader::open(filepath)?;

    let sample_rate = reader.spec().sample_rate;
    let duration = reader.duration();
    let bit_depth = reader.spec().bits_per_sample;
    let num_channels = reader.spec().channels;

    // Read samples as floats
    let samples: Vec<f32> = match reader.spec().sample_format {
        SampleFormat::Float => reader
            .into_samples::<f32>()
            .map(|sample| sample.unwrap())
            .collect(),
        SampleFormat::Int => {
            let max: i32 = match bit_depth {
                16 => i16::MAX as i32,
                24 => MAX_24BIT,
                32 => i32::MAX,
                _ => return Err(Box::new(UnsupportedBitDepth(bit_depth))),
            };
            reader
                .into_samples::<i32>()
                .map(|sample| sample.unwrap() as f32 / max as f32)
                .collect()
        }
    };

    // For stereo audio, only take samples from the left channel
    // TODO: Implement other methods to combine multiple channels
    // into one
    let samples = match num_channels {
        1 => samples,
        2 => {
            let mut left_samples = Vec::with_capacity(samples.len() / 2);

            // Samples are interleaved, samples from the left channel
            // are at even indices
            for sample in samples.iter().step_by(2) {
                left_samples.push(*sample);
            }
            left_samples
        }
        _ => return Err(Box::new(UnsupportedChannelCount(num_channels))),
    };

    let wav_file_data = WavFileData {
        sample_rate,
        duration,
        samples,
    };
    Ok(wav_file_data)
}

#[cfg(test)]
mod tests {
    use crate::wav_utils::read_wav_file;

    #[test]
    fn sample_rate_read_correctly() {
        let mut filepath = std::path::PathBuf::new();
        filepath.push("./resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

        let wav_file_data = read_wav_file(filepath).expect("Expected valid wav file data");
        assert_eq!(44100, wav_file_data.sample_rate);
    }

    #[test]
    fn duration_read_correctly() {
        let mut filepath = std::path::PathBuf::new();
        filepath.push("./resources/test/sine_660Hz_22050samples_s32bit_44100Hz_mono.wav");

        let wav_file_data = read_wav_file(filepath).expect("Expected valid wav file data");
        assert_eq!(22050, wav_file_data.duration);
    }

    #[test]
    fn reading_stereo_audio_works_correctly() {
        let mut filepath = std::path::PathBuf::new();
        filepath.push("./resources/test/sine_440Hz_44100samples_s16bit_44100Hz_stereo.wav");

        let wav_file_data = read_wav_file(filepath).expect("Expected valid wav file data");
        assert_eq!(44100, wav_file_data.sample_rate);
        assert_eq!(44100, wav_file_data.duration);
    }

    #[test]
    fn different_bit_depths_read_correctly() {
        let mut filepath_signed_16 = std::path::PathBuf::new();
        filepath_signed_16.push("./resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

        let wav_file_data_signed_16 =
            read_wav_file(filepath_signed_16).expect("Expected valid wav file data");

        let mut filepath_signed_24 = std::path::PathBuf::new();
        filepath_signed_24.push("./resources/test/sine_440Hz_44100samples_s24bit_44100Hz_mono.wav");

        let wav_file_data_signed_24 =
            read_wav_file(filepath_signed_24).expect("Expected valid wav file data");

        let mut filepath_float_32 = std::path::PathBuf::new();
        filepath_float_32.push("./resources/test/sine_440Hz_44100samples_f32bit_44100Hz_mono.wav");

        let wav_file_data_float_32 =
            read_wav_file(filepath_float_32).expect("Expected valid wav file data");
        assert_eq!(
            wav_file_data_float_32.sample_rate,
            wav_file_data_signed_16.sample_rate
        );
        assert_eq!(
            wav_file_data_float_32.sample_rate,
            wav_file_data_signed_24.sample_rate
        );
        assert_eq!(
            wav_file_data_float_32.duration,
            wav_file_data_signed_16.duration
        );
        assert_eq!(
            wav_file_data_float_32.duration,
            wav_file_data_signed_24.duration
        );
    }

    #[test]
    #[should_panic(expected = "Expected valid wav file data")]
    fn reading_non_wav_file_should_panic() {
        let mut filepath = std::path::PathBuf::new();
        filepath.push("./resources/test/README.md");

        read_wav_file(filepath).expect("Expected valid wav file data");
    }
}