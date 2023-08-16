# Olscorer

(Work in progress) Olscorer is an automatic music transcription program.

## Installation
Olscorer is written in the Rust programming language. To build the project, you will need to have Rust version 1.70.0+.

Run `cargo build --release` in the root directory to build the project in release mode and `cargo run` to build and run.

## Usage
To run Olscorer from the command line, run the `olscorer-cli` executable with the path to the wav file you would like to transcribe as an argument.
#### Input:
```
./olscorer-cli example_audio.wav
```
#### Output:
```
C5, E5, G5
```

The output is a comma-separated list of notes detected in the audio.

## Future Improvements

- A more useful CLI output. The current output is very minimal. In the future, a more useful output such as a MusicXML file could be used.
- GUI
- Polyphonic music transcription. Currently, the program can only transcribe monophonic music.

## Known Issues

- If a note is played immediately in the audio file, with no clear onset, it will not be recognized as a note.
- If consecutive notes overlap slightly, their pitches may not be detected correctly.

## Pitch Detection Methods

The pitch detection method the program currently uses is the "McLeod Pitch Method" described by Philip McLeod and Geoff Wyvill in their paper "A Smarter Way to Find Pitch" [1].

## License
Olscorer is licensed under GPL version 3.0 (or later). See the [LICENSE](./LICENSE) file.

## References
1. McLeod, Philip & Wyvill, Geoff. (2005). A smarter way to find pitch. [Link to Paper](https://quod.lib.umich.edu/i/icmc/bbp2372.2005.107/1/--smarter-way-to-find-pitch?rgn=full+text;view=image;q1=A+smarter+way+to+find+pitch)
