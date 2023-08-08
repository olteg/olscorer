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

use clap::Parser;
use olscorer::audio_utils::read_wav_file;
use olscorer::transcription::Transcriber;

#[derive(Debug, Parser)]
#[clap(name = "Olscorer", author, version, about)]
struct OlscorerArgs {
    /// Path to input audio file
    input_file: std::path::PathBuf,
}

fn main() {
    let args = OlscorerArgs::parse();

    let audio_data = read_wav_file(args.input_file).expect("Error reading wav file");

    // Get all the notes in the audio
    let all_notes = Transcriber::get_notes(audio_data);

    // Combine adjacent identical notes
    let mut notes = vec![];
    let mut prev_note_pos = 0;

    for note in all_notes {
        if notes.len() == 0 {
            notes.push(note);
        } else if notes[prev_note_pos].name != note.name {
            notes.push(note);
            prev_note_pos += 1;
        }
    }

    // Format and print notes as a comma-separated list
    let output_notes = notes
        .iter()
        .map(|note| note.name.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    println!("{}", output_notes);
}
