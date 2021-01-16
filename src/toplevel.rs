use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::cache::Cache;
use crate::clip::ClipProcess;
use crate::config::TrackData;
use crate::sox_args::SoxArgs;
use crate::track_name::TrackName;

pub fn clean_arg(matches: &clap::ArgMatches) {
    let track_name = TrackName::new_from_arg(matches);
    println!("Cleaning cache for track {}", track_name.get_name());
    std::fs::remove_dir_all(track_name.dest_dir()).ok(); // empty cache
}

pub fn export_arg(matches: &clap::ArgMatches) {
    println!("Exporting...");
    let track_name = TrackName::new_from_arg(matches);
    assert!(
        Command::new("cp")
            .arg(track_name.dest_dir().join(TrackData::processed_filename()))
            .arg(matches.value_of("output_file").unwrap())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("export command failed")
            .status
            .success(),
        "export command failed"
    );
}

pub fn play_arg(matches: &clap::ArgMatches) {
    println!("Playing via mpv...");
    let track_name = TrackName::new_from_arg(matches);
    assert!(
        Command::new("mpv")
            .arg(track_name.dest_dir().join(TrackData::processed_filename()))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("mpv command failed")
            .status
            .success(),
        "mpv command failed"
    );
}

pub fn build_arg(matches: &clap::ArgMatches) {
    build_track(TrackName::new_from_arg(matches));
}

pub fn get_tracks(matches: &clap::ArgMatches) -> Vec<TrackName> {
    fs::read_dir(
        matches
            .value_of("track_dir")
            .map(|dirname| PathBuf::from(dirname))
            .unwrap_or(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tracks")),
    )
    .unwrap()
    .map(|res| TrackName::new(res.map(|e| e.path()).unwrap().file_name().unwrap(), matches))
    .collect()
}

pub fn process_tracks(matches: &clap::ArgMatches) {
    for track_name in get_tracks(matches) {
        build_track(track_name);
    }
}

pub fn setup_directories(matches: &clap::ArgMatches) {
    println!("Creating build directories...");
    for dir in fs::read_dir(
        matches
            .value_of("track_dir")
            .map(|dirname| PathBuf::from(dirname))
            .unwrap_or(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tracks")),
    )
    .unwrap()
    .map(|res| res.map(|e| e.path()).unwrap())
    {
        let mut new_dir: PathBuf = ["target", "tracks"].iter().collect();
        new_dir.push(dir.file_name().unwrap());
        fs::create_dir_all(new_dir.into_os_string()).unwrap();
    }
}

pub fn build_track(track_name: TrackName) {
    println!("-> Building track {}", track_name);

    println!("--> Loading config file");
    let mut config = TrackData::load_from_track(&track_name);

    if let (Some(build_cfg), cache, updates) = (
        config.build().clone(),
        config.output().cache.unwrap_or(true),
        &mut config.updates,
    ) {
        // Check download/clone status
        build_cfg.create_dirs(&track_name);
        if build_cfg.git_sources.len() > 0 {
            println!("--> Downloading git sources");
            if build_cfg.git(&track_name) {
                updates.build_updated();
            }
        }
        if build_cfg.http_sources.len() > 0 {
            println!("--> Downloading http sources");
            if build_cfg.http(&track_name, cache) {
                updates.build_updated();
            }
        }
        if build_cfg.copy_me {
            println!("--> Copying local files");
            if build_cfg.local(&track_name, cache) {
                updates.build_updated();
            }
        }
        if updates.needs_build_update {
            build_cfg.write_cache(&track_name);
            build_cfg.wipe_build_progress(&track_name);
        }
        if build_cfg.build_command.len() > 0 {
            if build_cfg.run(&track_name) {
                updates.rebuilt();
            }
        }
    }

    if config.updates.needs_raw_update {
        config.output().write_cache(&track_name);
        println!(
            "--> Running output command and dumping {} of data",
            config.output().output_buffer
        );
        println!("---> {}", &config.output().output_command);
        config.dump_raw(&track_name);
    } else {
        println!("--> Output generation up to date; continuing");
    }

    if config.updates.needs_preprocessed_update {
        config.sox().write_cache(&track_name);

        println!("--> Piping through sox");
        SoxArgs::new(&track_name, &config).execute();
    } else {
        println!("--> Sox output up to date; continuing");
    }

    if config.updates.needs_ffmpeg_update {
        config.clips().write_cache(&track_name);
        config.clips().process(&track_name); // edit message is displayed internally, if required
    }

    println!("--> Finished processing track '{}'", config.output().name);
}
