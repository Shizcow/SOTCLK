mod sox_args;
mod track_name;
mod config;
mod build;
mod cache;
mod clip;
mod toplevel;

use clap::{Arg, App, SubCommand, AppSettings};

fn main() {
    toplevel::setup_directories();

    let matches = App::new("Sounds of the Compiling Linux Kernel")
        .version("1.0")
        .about("Interpreting interesting data as raw audio")
	.setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("build")
                    .about("Build a track, internally saving the result as a .flac file")
                    .arg(Arg::with_name("track")
			 .index(1)
			 .required(true)
                         .help("track directory name, found in tracks/")))
        .subcommand(SubCommand::with_name("build-all")
                    .about("Builds all tracks, internally saving results as .flac"))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("build") {
	let path = toplevel::get_tracks().into_iter().find(|tn| {
	    tn.get_name() == matches.value_of("track").unwrap()
	});
	if let Some(p) = path {
	    toplevel::build_track(p);
	} else {
	    panic!("Track '{}' not found in tracks/ directory", matches.value_of("track").unwrap());
	}
    } else if let Some(_) = matches.subcommand_matches("build-all") {
	println!("Building tracks...");
	toplevel::process_tracks();
    }

    println!("Done");
}
