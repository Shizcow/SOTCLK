use std::fs;
use std::path::PathBuf;

mod sox_args;
use sox_args::SoxArgs;
mod track_name;
use track_name::TrackName;
mod config;
use config::{TrackData, Cache};

fn main() {
    // create track directories
    println!("Creating build directories...");
    for dir in fs::read_dir("tracks").unwrap()
	.map(|res| res.map(|e| e.path()).unwrap()) {
	    let mut new_dir: PathBuf = ["target", "tracks"]
		.iter().collect();
	    new_dir.push(dir.file_name().unwrap());
	    fs::create_dir_all(new_dir.into_os_string()).unwrap();
	}

    
    println!("Building tracks...");
    for track_name in fs::read_dir("tracks").unwrap()
	.map(|res| TrackName::new(res.map(|e| e.path()).unwrap().file_name().unwrap())) {
	    
	    println!("-> Building track {}", track_name);
	    
	    println!("--> Loading config file");
	    let config = TrackData::load_from_track(&track_name);

	    if let Some(build_cfg) = &config.build() {
		println!("--> Checking build cache");
		if config.needs_build_update {
		    build_cfg.write_cache(&track_name);
		    build_cfg.create_dir(&track_name);
		    if build_cfg.build_command.len() > 0 {
			println!("--> Running build command");
			println!("---> {}", build_cfg.build_command);
			build_cfg.run(&track_name);
		    }
		} else {
		    println!("--> Build files up to date; continuing");
		}
	    }

	    println!("--> Checking raw cache");
	    if config.needs_raw_update {
		config.track().write_cache(&track_name);
		println!("--> Running output command and dumping data");
		println!("---> {}", &config.track().output_command);
		config.dump_raw(&track_name);
	    } else {
		println!("--> Output generation up to date; continuing");
	    }
	    
	    println!("--> Checking sox cache");
	    if config.needs_preprocessed_update {
		config.sox().write_cache(&track_name);
		
		println!("--> Piping through sox");
		SoxArgs::new(&track_name, &config).execute();
	    } else {
		println!("--> Sox output up to date; continuing");
	    }
	    
	    println!("--> Finished processing track '{}'", config.track().name);
	}

    println!("Done");
}
