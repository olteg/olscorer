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

const { invoke } = window.__TAURI__.tauri;
const { open, save, message } = window.__TAURI__.dialog;
const { BaseDirectory, writeTextFile } = window.__TAURI__.fs;
const { appWindow } = window.__TAURI__.window;

const openFileButton = document.getElementById("openFileButton");
const transcribeButton = document.getElementById("transcribeButton");
const saveButton = document.getElementById("saveFileButton");
const audioDataOutputText = document.getElementById("audioDataResultText");
const notesOutput = document.getElementById("notesResult");

async function openFile() {
    appWindow.setCursorIcon("wait");

    const selected = await open({
        multiple: false,
        filters: [
            {
                name: "wav",
                extensions: ["wav"],
            },
        ],
    });

    let selectedFile;

    if (Array.isArray(selected)) {
        selectedFile = selected[0];
    } else if (selected !== null) {
        selectedFile = selected;
    }

    if (selected !== null) {
        let returnValue;
        await invoke("get_audio_data", { filepath: selectedFile }).then(
            (response) => {
                const fileName = selectedFile
                    .split(selectedFile.includes("/") ? "/" : "\\")
                    .pop();
                audioDataOutputText.innerHTML = fileName;
                appWindow.setCursorIcon("default");
                returnValue = {
                    fileName: fileName,
                    audioData: response,
                };
            }
        );
        return returnValue;
    }
    appWindow.setCursorIcon("default");
}

async function transcribeAudio(audioData) {
    if (audioData === undefined) {
        await message("No audio to transcribe", { type: "error" });
    } else {
        let returnValue;
        appWindow.setCursorIcon("wait");
        await invoke("get_notes", { audioData: audioData }).then((response) => {
            appWindow.setCursorIcon("default");
            returnValue = response;
        });
        appWindow.setCursorIcon("default");
        return returnValue;
    }
}

async function saveNotes(notes, fileName) {
    if (fileName === undefined || notes === undefined) {
        await message("No transcription to save", { type: "error" });
    } else {
        const defaultFileName = fileName.split(".").shift() + ".txt";

        const filePath = await save({
            defaultPath: defaultFileName,
        });

        if (filePath !== null) {
            await writeTextFile({ path: filePath, contents: notes });
        }
    }
}

window.addEventListener("DOMContentLoaded", () => {
    let audioData;
    let notes;
    let fileName;

    openFileButton.addEventListener("click", (event) => {
        event.preventDefault();

        transcribeButton.setAttribute("disabled", "disabled");

        let audioDataPromise = openFile();

        audioDataPromise.then((response) => {
            if (response !== undefined) {
                fileName = response.fileName;
                audioData = response.audioData;
                transcribeButton.removeAttribute("disabled");
            }
        });
    });

    transcribeButton.addEventListener("click", (event) => {
        event.preventDefault();
        let notesPromise = transcribeAudio(audioData);

        notesPromise.then((response) => {
            notes = response;
            notesOutput.innerHTML = notes;
            if (response !== undefined) {
                saveButton.removeAttribute("disabled");
            }
        });
    });

    saveButton.addEventListener("click", (event) => {
        event.preventDefault();
        saveNotes(notes, fileName);
    });
});
