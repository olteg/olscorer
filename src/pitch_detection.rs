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

use rustfft::{num_complex::Complex, FftPlanner};

pub trait PitchDetector {
    /// Attempts to detect the pitch in the input samples
    ///
    /// If a pitch is detected, the frequency is returned, otherwise
    /// None is returned.
    fn get_pitch(self, samples: Vec<f64>) -> Option<f64>;
}

/// Struct for the McLeod Pitch Method pitch detection algorithm
#[derive(Clone)]
pub struct Mpm {
    threshold: f64,
    sample_rate: u32,
}

impl PitchDetector for Mpm {
    /// Attempts to detect the pitch in the input samples using the
    /// McLeod Pitch Method
    ///
    /// The method is described by Philip McLeod and Geoff Wyvill
    /// in "A Smarter Way to Find Pitch" (2005).
    ///
    /// If a pitch is detected, the frequency is returned, otherwise
    /// None is returned.
    fn get_pitch(self, samples: Vec<f64>) -> Option<f64> {
        let nsdf = Mpm::fast_nsdf(samples);

        match self.get_mpm_peak(nsdf) {
            Some(peak) => Some(self.sample_rate as f64 / peak.0),
            None => None,
        }
    }
}

impl Mpm {
    /// Creates a new McLeod Pitch Method pitch detector instance
    ///
    /// # Arguments
    ///
    /// * `threshold` - A coefficient used when deciding which pitch
    ///                 candidate to choose as the final estimate
    /// * `sample_rate` - The sample rate of the audio which the detector will
    ///                   be used on
    pub fn new(threshold: f64, sample_rate: u32) -> Mpm {
        Mpm {
            threshold,
            sample_rate,
        }
    }

    /// Calculates the normalized square difference function (NSDF) values,
    /// as described by Philip McLeod and Geoff Wyvill in
    /// "A Smarter Way to Find Pitch" (2005)
    ///
    /// The number of NSDF values calculated is equal to the number of samples.
    fn fast_nsdf(samples: Vec<f64>) -> Vec<f64> {
        let autoc_values = Mpm::fast_autoc(samples.clone());
        let sq_sums = Mpm::square_sums(samples.clone());

        let mut nsdf = vec![0.0; samples.len()];

        for tau in 0..samples.len() {
            let sq_sum = if sq_sums[tau] > f64::EPSILON {
                sq_sums[tau]
            } else {
                1.0
            };
            nsdf[tau] = 2.0 * autoc_values[tau] / sq_sum;
        }

        nsdf
    }

    /// Computes the autocorrelation of the input samples
    fn fast_autoc(samples: Vec<f64>) -> Vec<f64> {
        // Zero-pad the samples
        let fft_length = 2 * samples.len();
        let mut padded_samples = samples;
        padded_samples.resize(fft_length, 0.0);

        let mut planner = FftPlanner::new();
        let fft_forward = planner.plan_fft_forward(fft_length);

        let mut buffer: Vec<Complex<f64>> = padded_samples
            .iter()
            .map(|x| Complex { re: *x, im: 0.0 })
            .collect();

        fft_forward.process(&mut buffer);

        let scale_factor = 1.0 / (fft_length as f64).sqrt();
        let mut power_spectrum: Vec<Complex<f64>> =
            buffer.iter().map(|x| scale_factor * x * x.conj()).collect();

        let fft_inverse = planner.plan_fft_inverse(fft_length);

        fft_inverse.process(&mut power_spectrum);

        power_spectrum.iter().map(|x| scale_factor * x.re).collect()
    }

    /// Calculates the values used in the calculations of the normalized square
    /// difference function values
    ///
    /// The number of values calculated is equal to the number of samples.
    fn square_sums(samples: Vec<f64>) -> Vec<f64> {
        let mut sq_sums = vec![0.0; samples.len()];

        for tau in 0..samples.len() {
            let mut sq_sum = 0.0;
            for i in 0..samples.len() - tau {
                sq_sum += samples[i] * samples[i] + samples[i + tau] * samples[i + tau];
            }
            sq_sums[tau] = sq_sum;
        }
        sq_sums
    }

    /// Uses quadratic interpolation to estimate the position of a peak
    /// given three points with non-negative integer x-coordinates
    ///
    /// If the points form a straight line, or if two points have the same
    /// x-coordinate, None is returned
    fn quadratic_peak_interp(
        p0: (usize, f64),
        p1: (usize, f64),
        p2: (usize, f64),
    ) -> Option<(f64, f64)> {
        let (x0, y0) = (p0.0 as f64, p0.1);
        let (x1, y1) = (p1.0 as f64, p1.1);
        let (x2, y2) = (p2.0 as f64, p2.1);

        let div = (x0 - x1) * (x0 - x2) * (x1 - x2);

        // Return None if two points have the same x-coordinate
        if div.abs() < f64::EPSILON {
            return None;
        }

        // Calculate the coefficients of the quadratic equation
        let a = (y0 * (x1 - x2) + y1 * (x2 - x0) + y2 * (x0 - x1)) / div;

        // Return None if the points form a straight line
        if a.abs() < f64::EPSILON {
            return None;
        }

        let b =
            -(y0 * (x1 * x1 - x2 * x2) + y1 * (x2 * x2 - x0 * x0) + y2 * (x0 * x0 - x1 * x1)) / div;
        let c =
            (y0 * x1 * x2 * (x1 - x2) + y1 * x2 * x0 * (x2 - x0) + y2 * x0 * x1 * (x0 - x1)) / div;

        // Find the peak of the quadratic
        let x = -b / (2.0 * a);
        Some((x, a * x * x + b * x + c))
    }

    /// Peak picking algorithm described by Philip McLeod and Geoff Wyvill
    /// in "A Smarter Way to Find Pitch" (2005)
    fn get_mpm_peak(&self, nsdf: Vec<f64>) -> Option<(f64, f64)> {
        // Find first zero_crossing
        let mut start_index = 0;

        while start_index < nsdf.len() && nsdf[start_index] > 0.0 {
            start_index += 1;
        }

        let mut peaks: Vec<(f64, f64)> = vec![];

        let mut max_peak = (0.0, 0.0);
        // Only start looking for positive valued peaks after first zero-crossing
        let mut i = start_index;
        while i < nsdf.len() {
            // Skip negative values
            if nsdf[i] < 0.0 {
                i += 1;
                continue;
            }

            // Find local maximum between positive and negative zero crossings
            let mut interp_peak = (0.0, 0.0);
            let mut local_peak = (0.0, 0.0);

            while i < nsdf.len() && nsdf[i] >= 0.0 {
                if nsdf[i] > local_peak.1
                    && i > 0
                    && i < nsdf.len() - 1
                    && nsdf[i - 1] < nsdf[i]
                    && nsdf[i + 1] < nsdf[i]
                {
                    local_peak = (i as f64, nsdf[i]);
                    interp_peak = match Mpm::quadratic_peak_interp(
                        (i - 1, nsdf[i - 1]),
                        (i, nsdf[i]),
                        (i + 1, nsdf[i + 1]),
                    ) {
                        Some(peak) => peak,
                        None => interp_peak,
                    };
                    // Update global maximum peak if necessary
                    if local_peak.1 > max_peak.1 {
                        max_peak = local_peak;
                    }
                }
                i += 1;
            }

            if interp_peak.1 > 0.0 {
                peaks.push(interp_peak);
            }
            i += 1;
        }

        // Return the first peak above a certain threshold
        peaks
            .into_iter()
            .find(|x| x.1 > self.threshold * max_peak.1)
    }
}
