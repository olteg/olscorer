# Olscorer

(Work in progress) Olscorer is an automatic music transcription program.

## Installation
Olscorer is written in the Rust programming language. To build the project, you will need to have Rust version 1.70.0+.

Run `cargo build` in the root directory to build the project and `cargo run` to build and run.

## Pitch Detection Methods

The pitch detection method the program currently uses is the "McLeod Pitch Method" described by Philip McLeod and Geoff Wyvill in their paper "A Smarter Way to Find Pitch" [1].

## License
Olscorer is licensed under GPL version 3.0 (or later). See the [LICENSE](./LICENSE) file.

## References
1. McLeod, Philip & Wyvill, Geoff. (2005). A smarter way to find pitch. [Link to Paper](https://quod.lib.umich.edu/i/icmc/bbp2372.2005.107/1/--smarter-way-to-find-pitch?rgn=full+text;view=image;q1=A+smarter+way+to+find+pitch)
