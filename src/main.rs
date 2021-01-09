use std::fs;
use std::path::PathBuf;

mod sox_args;
use sox_args::SoxArgs;
mod track_name;
use track_name::TrackName;
mod config;
use config::{TrackConfig, SoxConfig};

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
	    let config = TrackConfig::load_from_track(&track_name);

	    println!("--> Checking build cache");
	    if SoxConfig::load_from_cache(&track_name) != Some(config.clone().into()) {
		config.write_cache(&track_name);
		println!("--> Running output command and dumping data");
		println!("---> {}", &config.track.output_command);

		config.dump_raw(&track_name);
		
		println!("--> Piping through sox");
		SoxArgs::new(&track_name, &config).execute();
	    } else {
		println!("--> Build files up to date; continuing");
	    }
	    
	    println!("--> Finished processing track '{}'", config.track.name);
	}

    println!("Done");
}
