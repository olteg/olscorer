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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WavFileError {
    #[error("unsupported bit depth `{0}`, expected 16, 24, or 32")]
    UnsupportedBitDepth(u16),

    #[error("unsupported channel count `{0}`, expected mono or stereo audio")]
    UnsupportedChannelCount(u16),
}
