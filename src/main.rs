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

fn main() {
    let track_arg = Arg::with_name("track")
        .index(1)
        .required(true)
        .help("track directory name, found in tracks/");
    let album_arg = Arg::with_name("album")
        .index(1)
        .required(true)
        .help("album config file name, found in albums/. Ex: 'ls'");
    let track_subcommand = SubCommand::with_name("track")
        .about("Build a track")
        .arg(track_arg.clone());
    let album_subcommand = SubCommand::with_name("album")
        .about("Build a track")
        .arg(album_arg.clone());
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
                .subcommand(track_subcommand.clone())
                .subcommand(album_subcommand.clone())
        )
        .subcommand(
            SubCommand::with_name("build-all")
                .about("Builds all tracks and albums, internally saving results as .flac"),
        )
        .subcommand(
            SubCommand::with_name("play")
                .about("Build and play a track using mpv")
                .arg(track_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("clean")
                .about("Wipe the cache of a track, triggering a rebuild")
		.setting(AppSettings::SubcommandRequired)
                .subcommand(track_subcommand.clone())
                .subcommand(album_subcommand.clone())
        )
        .subcommand(
            SubCommand::with_name("export")
                .about("Save a track's .flac somewhere")
                .arg(track_arg.clone())
                .arg(
                    Arg::with_name("output_file")
                        .index(2)
                        .required(true)
                        .help("Filename to save to. Ex: exported.flac"),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("clean") {
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
        } else if let Some(matches) = matches.subcommand_matches("play") {
            toplevel_track::build_arg(matches);
            toplevel_track::play_arg(matches);
        } else if let Some(matches) = matches.subcommand_matches("export") {
            toplevel_track::build_arg(matches);
            toplevel_track::export_arg(matches);
        }
    }

    println!("Done");
}
