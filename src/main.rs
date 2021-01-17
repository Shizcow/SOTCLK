mod album_data;
mod album_name;
mod build;
mod cache;
mod clip;
mod config;
mod sox_args;
mod toplevel_album;
mod toplevel_track;
mod track_name;

use clap::{App, AppSettings, Arg, SubCommand};
use std::fs;
use std::path::PathBuf;

fn main() {
    let track_arg = Arg::with_name("track")
        .index(1)
        .required(true)
        .help("track directory name, found in tracks/");
    let album_arg = Arg::with_name("album")
        .index(1)
        .required(true)
        .help("album config file name, found in albums/. Ex: 'ls'");
    let track_subcommand = SubCommand::with_name("track").arg(track_arg.clone());
    let album_subcommand = SubCommand::with_name("album").arg(album_arg.clone());
    let matches = App::new("Sounds of the Compiling Linux Kernel")
        .version("1.0")
        .about("Interpreting interesting data as raw audio")
        .setting(AppSettings::SubcommandRequired)
        .arg(
            Arg::with_name("track_dir")
                .long("--track-dir")
                .global(true)
                .takes_value(true)
                .value_name("TRACK_DIR")
                .help("Where to look for incoming track files. Defaults to git source directory."),
        )
        .arg(
            Arg::with_name("album_dir")
                .long("--album-dir")
                .global(true)
                .takes_value(true)
                .value_name("ALBUM_DIR")
                .help("Where to look for incoming album config files. Defaults to git source directory."),
        )
        .subcommand(
            SubCommand::with_name("build")
                .about("Build an item, internally saving the result as a .flac file or set of .flac files")
		.setting(AppSettings::SubcommandRequired)
                .subcommand(track_subcommand.clone()
			    .about("Build a track"))
                .subcommand(album_subcommand.clone()
			    .about("Build an album"))
        )
        .subcommand(
            SubCommand::with_name("build-all")
                .about("Builds all tracks and albums, internally saving results as .flac"),
        )
        .subcommand(
            SubCommand::with_name("play")
                .about("Build and play a track or album using mpv")
		.setting(AppSettings::SubcommandRequired)
                .subcommand(track_subcommand.clone().about("Play a track using mpv"))
                .subcommand(album_subcommand.clone().about("Build an album using mpv. Plays the compiled file"))
        )
        .subcommand(
            SubCommand::with_name("clean")
                .about("Wipe the cache of a track or album, triggering a rebuild")
		.setting(AppSettings::SubcommandRequired)
                .subcommand(track_subcommand.clone().about("Clean a track"))
                .subcommand(album_subcommand.clone().about("Clean an album, but not its individual tracks"))
        )
        .subcommand(
            SubCommand::with_name("clean-all")
                .about("Wipe all caches")
        )
        .subcommand(
            SubCommand::with_name("export")
                .about("Save a track's .flac somewhere, or export an entire album's directory")
                .subcommand(track_subcommand.clone().arg(
                    Arg::with_name("output_file")
                        .index(2)
                        .required(true)
                        .help("Filename to save to. Ex: exported.flac")
                )
			.about("Export a .flac somewhere"))
                .subcommand(album_subcommand.clone().arg(
                    Arg::with_name("output_dir")
                        .index(2)
                        .required(true)
                        .help("Directory to save to. A subdirectory with all the album content will be created")
                )
			    .about("Export the entire album directory. Includes individual tracks, a compiled master-track, and tracklist with some additional info"))
        )
        .get_matches();

    if let Some(_) = matches.subcommand_matches("clean-all") {
        fs::remove_dir_all(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("tracks"),
        )
        .unwrap();
        fs::remove_dir_all(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("albums"),
        )
        .unwrap();
    } else if let Some(matches) = matches.subcommand_matches("clean") {
        if let Some(matches) = matches.subcommand_matches("track") {
            toplevel_track::clean_arg(matches);
        } else if let Some(matches) = matches.subcommand_matches("album") {
            toplevel_album::clean_arg(matches);
        }
    } else {
        toplevel_track::setup_directories(&matches);
        if let Some(matches) = matches.subcommand_matches("build") {
            if let Some(matches) = matches.subcommand_matches("track") {
                toplevel_track::build_arg(matches);
            } else if let Some(matches) = matches.subcommand_matches("album") {
                toplevel_album::build_arg(matches);
            }
        } else if let Some(matches) = matches.subcommand_matches("build-all") {
            println!("Building tracks...");
            toplevel_track::process_tracks(matches);
            println!("Building albums...");
            toplevel_album::process_albums(matches);
        } else if let Some(matches) = matches.subcommand_matches("play") {
            if let Some(matches) = matches.subcommand_matches("track") {
                toplevel_track::build_arg(matches);
                toplevel_track::play_arg(matches);
            } else if let Some(matches) = matches.subcommand_matches("album") {
                toplevel_album::build_arg(matches);
                toplevel_album::play_arg(matches);
            }
        } else if let Some(matches) = matches.subcommand_matches("export") {
            if let Some(matches) = matches.subcommand_matches("track") {
                toplevel_track::build_arg(matches);
                toplevel_track::export_arg(matches);
            } else if let Some(matches) = matches.subcommand_matches("album") {
                toplevel_album::build_arg(matches);
                toplevel_album::export_arg(matches);
            }
        }
    }

    println!("Done");
}
