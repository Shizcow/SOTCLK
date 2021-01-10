use std::path::PathBuf;
use std::fs;

use crate::sox_args::SoxArgs;
use crate::config::TrackData;
use crate::track_name::TrackName;
use crate::cache::Cache;
use crate::clip::ClipProcess;

fn get_tracks() -> Vec<TrackName> {
    fs::read_dir("tracks").unwrap()
	.map(|res| TrackName::new(res.map(|e| e.path()).unwrap().file_name().unwrap()))
	.collect()
}

pub fn process_tracks() {
    for track_name in get_tracks() {
	    build_track(track_name);
	}
}

pub fn setup_directories() {
    println!("Creating build directories...");
    for dir in fs::read_dir("tracks").unwrap()
	.map(|res| res.map(|e| e.path()).unwrap()) {
	    let mut new_dir: PathBuf = ["target", "tracks"]
		.iter().collect();
	    new_dir.push(dir.file_name().unwrap());
	    fs::create_dir_all(new_dir.into_os_string()).unwrap();
	}
}

pub fn build_track(track_name: TrackName) {
    println!("-> Building track {}", track_name);
    
    println!("--> Loading config file");
    let mut config = TrackData::load_from_track(&track_name);

    if let (Some(build_cfg), cache, updates) = (config.build().clone(),
						config.output().cache.unwrap_or(true),
						&mut config.updates) {
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
	if updates.needs_build_update {
	    build_cfg.write_cache(&track_name);
	    build_cfg.wipe_build_progress(&track_name);
	}
	if build_cfg.build_command.len() > 0 {
	    println!("--> Building");
	    if build_cfg.run(&track_name) {
		updates.rebuilt();
	    }
	}
    }

    if config.updates.needs_raw_update {
	config.output().write_cache(&track_name);
	println!("--> Running output command and dumping {} of data",
		 config.output().output_buffer);
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
	println!("--> Editing with ffmpeg");
	config.clips().process(&track_name);
    }
    
    println!("--> Finished processing track '{}'", config.output().name);
}