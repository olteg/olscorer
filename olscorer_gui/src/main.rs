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

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use olscorer_core::audio_utils::AudioData;
use olscorer_core::transcription::Transcriber;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_notes, get_audio_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_notes(audio_data: AudioData) -> String {
    Transcriber::get_notes(audio_data)
        .iter()
        .map(|note| note.name.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}

#[tauri::command]
fn get_audio_data(filepath: &str) -> AudioData {
    let mut path = std::path::PathBuf::new();
    path.push(filepath);
    AudioData::read_wav_file(path).expect("Error reading wav file")
}
