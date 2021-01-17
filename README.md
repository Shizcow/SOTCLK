## What?
### Previous Work
This project is a continuation of the [Sounds of the Compiling Linux Kernel album](https://www.youtube.com/watch?v=4yMVkQRhiiQ).
When I first made that album, nothing was documented and building was a long, slow, and tedious manual process.
This repo serves as an infinitely better way to create tracks.
### How it works
`sox(1)` is an amazing program that can, among other things, interpret audio streams
without any header information. This feature is often used to convert and play audio
mid-stream. It also does a pretty good job of converting raw data into audio. While
the results are far from what can be considered music, interesting sounds can pop up
in the most unlikely places surprisingly often.

Data used can be program output, file contents, or really anything that can be read from
a file. So long as there's enough data, `sox` can get something meaningful.

## Building and Running

### Building Source
This project is written in Rust, and thus requires a Rust compiler. To build the project,
run `cargo build` from the project's root directory.

### Building Tracks
From the root directory, run the following command:
```sh
cargo run -- build-all
```
This will build all tracks in the [`tracks/`](tracks) directory. This may include downloading
external files, cloning git repositories, and executing build commands. This step also takes
care of `sox` and `ffmpeg` processing. Built tracks are stored internally; see the following
section for playing.

Single tracks can also be built with the following command:
```sh
cargo run -- build track TRACK_NAME
```
Where `TRACK_NAME` is the directory name of a track in the [`tracks/`](tracks) directory.
For example, to build the `ls` track, run `cargo run -- build track ls`.

### Playing Tracks
Playing a track is done through the following command:
```sh
cargo run -- play track TRACK_NAME
```
Where `TRACK_NAME` is the directory name of a track in the [`tracks/`](tracks) directory.
For example, to play the `ls` track, run `cargo run -- play track ls`.

Playing a track through the `play` subcommand also builds that track.

If `mpv` is not available, you can export the track (see below) and play through your desired
media player.

### Exporting Tracks
Use the following command to export a track:
```sh
cargo run -- export track TRACK_NAME FILENAME.flac
```
Where `TRACK_NAME` is the directory name of a track in the [`tracks/`](tracks) directory,
and where `FILENAME.flac` is the desired __output__ filename, relative to the local
directory.

For example, to build and export the `ls` track to `ls.flac`,
run `cargo run -- export track ls ls.flac`.

### Albums
Albums have the same commands and syntaxes as tracks. For example:
```sh
cargo run -- play album ALBUM_NAME
```

When exporting a album, only give the destination directory. Album exports contain a lot more than one file, so an entire directory is generated to hold everything. This exported data includes:  
- An `individual-tracks` folder holding carbon-coppies of each track
- A master-track/compilation of all tracks in order
- A `tracklist.txt` with timestamps

## Creating/Configuring Tracks
New tracks can be added for compilation through the following steps:  
- Creating a new directory in the [`tracks/`](tracks) directory
- Creating a `config.toml` file in the new directory

After that, the tracks can be built via the commands in the
[Building Tracks](#building-tracks) section.

For a complete example `config.toml` with documentation, see
[`sample_config.toml`](sample_config.toml).

## Creating/Configuring Albums
It's simple enough; just look in the [`albums/`](albums) folder for examples. Really not a lot of configuring to do.

## Error Checking
There isn't any. Considering that takes time and this is a __really__ dumb project,
there probably won't be. Unless, of course, an issue is filed -- I'm happy to help
diagnose problems and introduce error checking if people actually end up using this.

If you don't like the messy code -- fix it yourself and submit a PR.

## Dependencies
The following dependencies are required for building:  
- `rustc`, `cargo`, etc
- `sh`
- `sox`
- `ffmpeg`
- `cp`
- `head`

The following dependencies are required during building if a project specifies sources:  
- `curl` and `libcurl` for http sources
- `git` for git sources

`mpv` is also required to play files through `cargo`. Options for other exporting and
using other players will come later.
