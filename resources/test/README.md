# Test Files
This folder contains files which are used to test the program.

## Example Files
- `example_audio.wav` - Simple piano example with the notes C5, E5, and G5 played in ascending order.
- `twinkle_twinkle_little_star.wav` - Twinkle Twinkle Little Star melody.
- `la_campanella.wav` - Melody from La Campanella by Liszt.
- `symphony_no_5.wav` - Short excerpt from Beethoven's Symphony No. 5.
- `piano_C_Major_scale.wav` - Ascending C Major scale in quarter notes at 144 bpm, starting at C5.

## Naming Conventions
Files in this directory follow a specific naming convention based on the type of the their contents.
### Sine Wave Files
Sine wave files should follow the following naming convention:

`sine_<frequency>Hz_<samples>samples_[f|s|u]<bit depth>bit_<sample rate>Hz_<'mono'|'stereo'>.wav`

For example:

`sine_440Hz_22050samples_s16bit_44100Hz_stereo.wav`

`sine_220Hz_44100samples_u32bit_44100Hz_mono.wav`

## Notes
The sine wave test files have been generated using Audacity.

The audio files featuring instruments are generated using Musescore 3.
