mod sox_args;
mod track_name;
mod config;
mod build;
mod cache;
mod clip;
mod toplevel;

use clap::{Arg, App, SubCommand, AppSettings};

fn main() {
    let track_arg = Arg::with_name("track")
	.index(1)
	.required(true)
        .help("track directory name, found in tracks/");
    let matches = App::new("Sounds of the Compiling Linux Kernel")
	.version("1.0")
	.about("Interpreting interesting data as raw audio")
	.setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("build")
                    .about("Build a track, internally saving the result as a .flac file")
                    .arg(track_arg.clone()))
        .subcommand(SubCommand::with_name("build-all")
                    .about("Builds all tracks, internally saving results as .flac"))
        .subcommand(SubCommand::with_name("play")
                    .about("Build and play a track using mpv")
                    .arg(track_arg.clone()))
        .subcommand(SubCommand::with_name("clean")
                    .about("Wipe the cache of a track, triggering a rebuild")
                    .arg(track_arg.clone()))
        .subcommand(SubCommand::with_name("export")
                    .about("Save a track's .flac somewhere")
                    .arg(track_arg.clone())
		    .arg(Arg::with_name("output_file")
			 .index(2)
			 .required(true)
			 .help("Filename to save to. Ex: exported.flac")))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("clean") {
	toplevel::clean_arg(matches);
    } else {
	toplevel::setup_directories();
	if let Some(matches) = matches.subcommand_matches("build") {
	    toplevel::build_arg(matches);
	} else if let Some(_) = matches.subcommand_matches("build-all") {
	    println!("Building tracks...");
	    toplevel::process_tracks();
	} else if let Some(matches) = matches.subcommand_matches("play") {
	    toplevel::build_arg(matches);
	    toplevel::play_arg(matches);
	} else if let Some(matches) = matches.subcommand_matches("export") {
	    toplevel::build_arg(matches);
	    toplevel::export_arg(matches);
	}
    }

    println!("Done");
}
