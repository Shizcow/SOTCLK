In progress! See [the first album](https://www.youtube.com/watch?v=4yMVkQRhiiQ) for an idea of what this project is about.

## Workflow
Below is a draft workflow of how this will work:
- Have a `src` folder for source code and a `tracks` folder for configuration
- Each track consists of a `config.toml` configuration file placed in a `trackname` directory
- Each configuration file specifies setup commands, output generation commands, and audio processing
- A build script (cargo) is able to build each track
- A build subscript (cargo run) is able to run each compiled track
- File regeneration is tracked through file edit times, similar to GNU Make

Stretch goals include:
- A build subscript is able to compile tracks into full albums
  - Consider a `albums` folder, similar to the `tracks` folder
- Albums and tracks can generate background video of command output as rendered in a terminal emulator
