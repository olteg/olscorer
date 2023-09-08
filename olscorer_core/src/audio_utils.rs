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

use crate::errors::{
    FrameError::{DuplicateFrameIndices, FrameIndexOutOfBounds, FrameIndicesNotSorted},
    WavFileError::{UnsupportedBitDepth, UnsupportedChannelCount},
};
use hound::{SampleFormat, WavReader};
use std::error::Error;

const MAX_24BIT: i32 = 16777215;

#[derive(Debug, PartialEq)]
pub struct Frame {
    /// Starting position of this frame in the original audio
    pub start_pos: usize,
    /// Samples contained in this frame
    pub samples: Vec<f64>,
}

pub struct AudioData {
    /// Sample rate (in Hz)
    pub sample_rate: u32,
    /// Duration (in samples) of the wav file
    pub duration: u32,
    pub samples: Vec<f64>,
}

impl AudioData {
    /// Returns a vector of audio frames from the samples in the AudioData struct
    ///
    /// # Arguments
    ///
    /// * `frame_width` - Number of samples each frame should contain
    /// * `step_size` - Interval between starting position of consecutive frames
    /// * `start` - (Optional) First frame should start at this sample
    /// * `end` - (Optional) Final frame should end at, but not include, this sample
    pub fn get_frames(
        &self,
        frame_width: usize,
        step_size: usize,
        start: Option<usize>,
        end: Option<usize>,
    ) -> Vec<Frame> {
        assert_ne!(0, frame_width, "frame width must be non-negative");

        let start = start.unwrap_or(0);
        let end = std::cmp::min(end.unwrap_or(self.samples.len()), self.samples.len());

        let mut frames = Vec::new();

        for i in (start..=(end - frame_width)).step_by(step_size) {
            let mut frame_samples = vec![0.0; frame_width as usize];
            frame_samples.clone_from_slice(&self.samples[i..i + frame_width]);
            let frame = Frame {
                start_pos: i,
                samples: frame_samples.into_iter().map(|s| s as f64).collect(),
            };

            frames.push(frame);
        }

        frames
    }

    /// Gets the audio frames which start at the given indices
    ///
    /// Returns a vector of audio frames wrapped in Ok if there are no errors,
    /// and an Error otherwise.
    pub fn get_frames_by_index(&self, indices: Vec<usize>) -> Result<Vec<Frame>, Box<dyn Error>> {
        // Check that indices vector is sorted in ascending order
        if !indices.windows(2).all(|w| w[0] <= w[1]) {
            return Err(Box::new(FrameIndicesNotSorted()));
        }

        let mut frames = Vec::new();

        if indices.is_empty() {
            return Ok(frames);
        }

        for i in 0..indices.len() {
            if indices[i] >= self.samples.len() {
                return Err(Box::new(FrameIndexOutOfBounds(indices[i])));
            }

            if i < indices.len() - 1 {
                if indices[i + 1] >= self.samples.len() {
                    return Err(Box::new(FrameIndexOutOfBounds(indices[i + 1])));
                }

                if indices[i] == indices[i + 1] {
                    return Err(Box::new(DuplicateFrameIndices(indices[i], i, i + 1)));
                }

                let mut frame_samples = vec![0.0; indices[i + 1] - indices[i] as usize];
                frame_samples.clone_from_slice(&self.samples[indices[i]..indices[i + 1]]);

                frames.push(Frame {
                    start_pos: indices[i],
                    samples: frame_samples,
                });
            } else {
                // Push the frame starting at the last index
                frames.push(Frame {
                    start_pos: indices[indices.len() - 1],
                    samples: self.samples[indices[indices.len() - 1]..self.samples.len()].to_vec(),
                });
            }
        }

        Ok(frames)
    }

    /// Gets the audio data from a wav file
    ///
    /// Returns an AudioData struct wrapped in Ok if there are no errors, and
    /// an Error otherwise.
    pub fn read_wav_file(filepath: std::path::PathBuf) -> Result<AudioData, Box<dyn Error>> {
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
        }
        .into_iter()
        .map(|s| s as f64)
        .collect();

        let audio_data = AudioData {
            sample_rate,
            duration,
            samples,
        };
        Ok(audio_data)
    }

    /// Calculates the root mean square of the input samples
    ///
    /// Returns the root mean square wrapped in Some if the samples vector is
    /// non-empty, otherwise returns None.
    pub fn root_mean_square(samples: Vec<f64>) -> Option<f64> {
        if samples.len() == 0 {
            return None;
        }
        Some((samples.iter().map(|x| x * x).sum::<f64>() / samples.len() as f64).sqrt())
    }
}

#[cfg(test)]
mod tests {
    /// Tests for wav file reader
    mod read_wav_file_tests {
        use crate::audio_utils::AudioData;

        #[test]
        fn sample_rate_read_correctly() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

            let audio_data =
                AudioData::read_wav_file(filepath).expect("Expected valid wav file data");
            assert_eq!(44100, audio_data.sample_rate);
        }

        #[test]
        fn duration_read_correctly() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/sine_660Hz_22050samples_s32bit_44100Hz_mono.wav");

            let audio_data =
                AudioData::read_wav_file(filepath).expect("Expected valid wav file data");
            assert_eq!(22050, audio_data.duration);
        }

        #[test]
        fn reading_stereo_audio_works_correctly() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_stereo.wav");

            let audio_data =
                AudioData::read_wav_file(filepath).expect("Expected valid wav file data");
            assert_eq!(44100, audio_data.sample_rate);
            assert_eq!(44100, audio_data.duration);
        }

        #[test]
        fn different_bit_depths_read_correctly() {
            let mut filepath_signed_16 = std::path::PathBuf::new();
            filepath_signed_16
                .push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

            let audio_data_signed_16 =
                AudioData::read_wav_file(filepath_signed_16).expect("Expected valid wav file data");

            let mut filepath_signed_24 = std::path::PathBuf::new();
            filepath_signed_24
                .push("../resources/test/sine_440Hz_44100samples_s24bit_44100Hz_mono.wav");

            let audio_data_signed_24 =
                AudioData::read_wav_file(filepath_signed_24).expect("Expected valid wav file data");

            let mut filepath_float_32 = std::path::PathBuf::new();
            filepath_float_32
                .push("../resources/test/sine_440Hz_44100samples_f32bit_44100Hz_mono.wav");

            let audio_data_float_32 =
                AudioData::read_wav_file(filepath_float_32).expect("Expected valid wav file data");
            assert_eq!(
                audio_data_float_32.sample_rate,
                audio_data_signed_16.sample_rate
            );
            assert_eq!(
                audio_data_float_32.sample_rate,
                audio_data_signed_24.sample_rate
            );
            assert_eq!(audio_data_float_32.duration, audio_data_signed_16.duration);
            assert_eq!(audio_data_float_32.duration, audio_data_signed_24.duration);
        }

        #[test]
        fn different_sample_rates_read_correctly() {
            let mut filepath_8000 = std::path::PathBuf::new();
            filepath_8000.push("../resources/test/sine_440Hz_8000samples_s16bit_8000Hz_mono.wav");

            let audio_data_8000 =
                AudioData::read_wav_file(filepath_8000).expect("Expected valid wav file data");

            assert_eq!(8000, audio_data_8000.sample_rate);

            let mut filepath_22050 = std::path::PathBuf::new();
            filepath_22050.push("../resources/test/sine_440Hz_22050samples_s16bit_22050Hz_mono.wav");

            let audio_data_22050 =
                AudioData::read_wav_file(filepath_22050).expect("Expected valid wav file data");

            assert_eq!(22050, audio_data_22050.sample_rate);

            let mut filepath_44100 = std::path::PathBuf::new();
            filepath_44100.push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

            let audio_data_44100 =
                AudioData::read_wav_file(filepath_44100).expect("Expected valid wav file data");

            assert_eq!(44100, audio_data_44100.sample_rate);
            // Duration (in seconds) should be the same for all 3 files
            // duration in seconds = duration in samples / sample rate
            assert_eq!(
                audio_data_8000.duration / audio_data_8000.sample_rate,
                audio_data_44100.duration / audio_data_44100.sample_rate
            );
            assert_eq!(
                audio_data_22050.duration / audio_data_22050.sample_rate,
                audio_data_44100.duration / audio_data_44100.sample_rate
            );
        }

        #[test]
        #[should_panic(expected = "Expected valid wav file data")]
        fn reading_non_wav_file_should_panic() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/README.md");

            AudioData::read_wav_file(filepath).expect("Expected valid wav file data");
        }
    }

    /// Tests for AudioData methods
    mod get_frames_tests {
        use crate::audio_utils::AudioData;

        #[test]
        fn returns_correct_number_of_frames() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

            let audio_data =
                AudioData::read_wav_file(filepath).expect("Expected valid wav file data");

            assert_eq!(10, audio_data.get_frames(4410, 4410, None, None).len());
            // Expected number of frames is 19 since the 20th frame starting at
            // sample 41895 would not have width 4410, as there are only 44100
            // samples in total
            assert_eq!(19, audio_data.get_frames(4410, 2205, None, None).len());
            assert_eq!(20, audio_data.get_frames(2205, 2205, None, None).len());
            assert_eq!(
                audio_data.samples.len(),
                audio_data.get_frames(1, 1, None, None).len()
            );
            assert_eq!(
                10,
                audio_data.get_frames(2205, 2205, Some(22050), None).len()
            );
            assert_eq!(
                1,
                audio_data.get_frames(2205, 2205, Some(41895), None).len()
            );
            assert_eq!(
                10,
                audio_data.get_frames(2205, 2205, None, Some(22050)).len()
            );
            assert_eq!(1, audio_data.get_frames(2205, 2205, None, Some(2205)).len());
            assert_eq!(
                0,
                audio_data
                    .get_frames(2205, 2205, Some(22050), Some(22050))
                    .len()
            );
            assert_eq!(
                0,
                audio_data
                    .get_frames(2205, 2205, Some(22051), Some(22050))
                    .len()
            );
        }

        #[test]
        fn overlapping_frame_contents_are_consistent() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

            let audio_data =
                AudioData::read_wav_file(filepath).expect("Expected valid wav file data");
            let frames = audio_data.get_frames(4410, 2205, None, None);

            assert_eq!(frames[0].samples[2205], frames[1].samples[0]);
            assert_eq!(frames[0].samples[4409], frames[1].samples[2204]);
            assert_eq!(frames[1].samples[2205], frames[2].samples[0]);
            assert_eq!(frames[1].samples[4409], frames[2].samples[2204]);
        }

        #[test]
        fn different_starting_point_frame_contents_are_consistent() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

            let audio_data =
                AudioData::read_wav_file(filepath).expect("Expected valid wav file data");
            let frames_1 = audio_data.get_frames(4410, 2205, None, None);
            let frames_2 = audio_data.get_frames(4410, 2205, Some(2205), None);

            assert_eq!(frames_1[0].samples[2205], frames_2[0].samples[0]);
            assert_eq!(frames_1[0].samples[4400], frames_2[0].samples[2195]);
            assert_eq!(frames_1[1].samples[2205], frames_2[1].samples[0]);
            assert_eq!(frames_1[1].samples[4400], frames_2[1].samples[2195]);
        }

        #[test]
        #[should_panic]
        fn zero_step_size_should_panic() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

            let audio_data =
                AudioData::read_wav_file(filepath).expect("Expected valid wav file data");
            audio_data.get_frames(4410, 0, None, None);
        }

        #[test]
        #[should_panic(expected = "frame width must be non-negative")]
        fn zero_frame_width_should_panic() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

            let audio_data =
                AudioData::read_wav_file(filepath).expect("Expected valid wav file data");
            audio_data.get_frames(0, 4410, None, None);
        }

        #[test]
        fn end_point_greater_than_number_of_samples_should_work_correctly() {
            let mut filepath = std::path::PathBuf::new();
            filepath.push("../resources/test/sine_440Hz_44100samples_s16bit_44100Hz_mono.wav");

            let audio_data =
                AudioData::read_wav_file(filepath).expect("Expected valid wav file data");

            // If the end point is greater than the number of samples, the frames
            // should cover all the samples, as if no end point was set
            assert_eq!(
                10,
                audio_data.get_frames(4410, 4410, None, Some(50000)).len()
            );
        }
    }

    mod get_frames_by_index_tests {
        use crate::audio_utils::AudioData;

        #[test]
        fn returns_correct_number_of_frames() {
            let samples = vec![0.0; 10];

            let indices1 = vec![0, 2, 4, 6, 8];
            let indices2 = vec![0, 4, 8];
            let indices3 = vec![4, 8];

            let audio_data = AudioData {
                sample_rate: 44100,
                duration: samples.len() as u32,
                samples: samples,
            };

            assert_eq!(5, audio_data.get_frames_by_index(indices1).unwrap().len());
            assert_eq!(3, audio_data.get_frames_by_index(indices2).unwrap().len());
            assert_eq!(2, audio_data.get_frames_by_index(indices3).unwrap().len());
        }

        #[test]
        fn no_indices_returns_empty_vector() {
            let samples = vec![0.0; 10];

            let indices: Vec<usize> = vec![];

            let audio_data = AudioData {
                sample_rate: 44100,
                duration: samples.len() as u32,
                samples: samples,
            };

            assert_eq!(0, audio_data.get_frames_by_index(indices).unwrap().len());
        }

        #[test]
        fn index_out_of_bounds_returns_error() {
            let samples = vec![0.0; 10];

            let indices: Vec<usize> = vec![20];

            let audio_data = AudioData {
                sample_rate: 44100,
                duration: samples.len() as u32,
                samples: samples,
            };

            assert_eq!(
                "index `20` is out of bounds",
                audio_data
                    .get_frames_by_index(indices)
                    .unwrap_err()
                    .to_string()
            );
        }

        #[test]
        fn duplicate_indices_returns_error() {
            let samples = vec![0.0; 10];

            let indices1: Vec<usize> = vec![0, 0];
            let indices2: Vec<usize> = vec![0, 1, 1];

            let audio_data = AudioData {
                sample_rate: 44100,
                duration: samples.len() as u32,
                samples: samples,
            };

            assert_eq!(
                "duplicate index `0` at positions 0 and 1 in `indices`",
                audio_data
                    .get_frames_by_index(indices1)
                    .unwrap_err()
                    .to_string()
            );
            assert_eq!(
                "duplicate index `1` at positions 1 and 2 in `indices`",
                audio_data
                    .get_frames_by_index(indices2)
                    .unwrap_err()
                    .to_string()
            );
        }

        #[test]
        fn indices_vector_must_be_sorted() {
            let samples = vec![0.0; 10];

            let indices: Vec<usize> = vec![3, 2, 1];

            let audio_data = AudioData {
                sample_rate: 44100,
                duration: samples.len() as u32,
                samples: samples,
            };

            assert_eq!(
                "`indices` must be sorted in ascending order",
                audio_data
                    .get_frames_by_index(indices)
                    .unwrap_err()
                    .to_string()
            )
        }

        #[test]
        fn frames_contain_correct_samples() {
            let samples: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0, 4.0];

            let indices1: Vec<usize> = vec![0, 1, 2, 3, 4];
            let indices2: Vec<usize> = vec![0, 2, 4];

            let audio_data = AudioData {
                sample_rate: 44100,
                duration: samples.len() as u32,
                samples: samples,
            };

            let frames1 = audio_data.get_frames_by_index(indices1).unwrap();
            let frames2 = audio_data.get_frames_by_index(indices2).unwrap();

            assert_eq!(0.0, frames1[0].samples[0]);
            assert_eq!(1.0, frames1[1].samples[0]);
            assert_eq!(2.0, frames1[2].samples[0]);
            assert_eq!(3.0, frames1[3].samples[0]);
            assert_eq!(4.0, frames1[4].samples[0]);

            assert_eq!(0.0, frames2[0].samples[0]);
            assert_eq!(1.0, frames2[0].samples[1]);
            assert_eq!(2.0, frames2[1].samples[0]);
            assert_eq!(3.0, frames2[1].samples[1]);
            assert_eq!(4.0, frames2[2].samples[0]);
        }
    }

    mod root_mean_square_tests {
        use crate::audio_utils::AudioData;

        #[test]
        fn root_mean_square_works_correctly() {
            assert_eq!(0.0, AudioData::root_mean_square(vec![0.0]).unwrap());
            assert_eq!(1.0, AudioData::root_mean_square(vec![1.0, 1.0]).unwrap());
            assert_eq!(1.0, AudioData::root_mean_square(vec![-1.0, 1.0]).unwrap());
            assert_eq!(
                1.0,
                AudioData::root_mean_square(vec![-1.0, 1.0, 1.0]).unwrap()
            );
            assert_eq!(
                10.0,
                AudioData::root_mean_square(vec![-10.0, -10.0]).unwrap()
            );
            assert!(
                (5.0 * 10.0f64.sqrt())
                    - AudioData::root_mean_square(vec![-10.0, 20.0])
                        .unwrap()
                        .abs()
                    < 1e-12
            );
        }

        #[test]
        fn root_mean_square_of_empty_vector_is_none() {
            assert_eq!(None, AudioData::root_mean_square(vec![]));
        }
    }
}
