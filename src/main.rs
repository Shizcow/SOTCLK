#![feature(command_access)]

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs::{self, File};
use std::io::Write;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;


#[derive(Serialize, Deserialize, Debug)]
struct TrackConfig {
    track: Track,
    sox: Sox,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sox {
    bit_depth: u32,
    sample_rate: u32,
    channels: u32,
    encoding: String,
    other_options: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Track {
    name: String,
    build_command: Option<String>,
    output_command: String,
    output_buffer: String,
}

fn main() {
    // create track directories
    println!("Creating build directories...");
    fs::create_dir_all("target/tracks").unwrap(); // TODO cargo root
    for dir in fs::read_dir("tracks").unwrap()
	.map(|res| res.map(|e| e.path()).unwrap()) {
	    let mut new_dir_name = OsString::new();
	    new_dir_name.push(OsStr::new("target/tracks/"));
	    new_dir_name.push(dir.file_name().unwrap());
	    fs::create_dir_all(new_dir_name).unwrap();
	}

    
    println!("Building tracks...");
    for track_name in fs::read_dir("tracks").unwrap()
	.map(|res| res.map(|e| e.path()).unwrap().file_name().unwrap().to_owned()){
	    
	    println!("-> Building track {}",
		     String::from_utf8_lossy(track_name.as_bytes()));
	    
	    let mut source_dir_name: OsString = "tracks/".to_owned().into();
	    source_dir_name.push(&track_name);

	    
	    println!("--> Loading config file");
	    
	    // load a toml file
	    let mut config_file = OsString::new();
	    config_file.push(source_dir_name);
	    config_file.push(OsStr::new("/config.toml"));
	    
	    let config: TrackConfig = toml::from_str(
		&fs::read_to_string(config_file).unwrap()
	    ).unwrap();

	    println!("--> Checking build cache");

	    // Check if output needs to be regenerated
	    let mut sox_config_cache: OsString = "target/tracks/".to_owned().into();
	    sox_config_cache.push(&track_name);
	    sox_config_cache.push(OsStr::new("/sox.toml"));
	    
	    let current_sox_config_str = format!(
		"[track]\n{}\n[sox]\n{}",
		toml::to_string(&config.track).unwrap(),
		toml::to_string(&config.sox).unwrap());
	    let old_sox_config_str = &fs::read_to_string(&sox_config_cache)
		.unwrap_or("".to_owned());

	    if &current_sox_config_str != old_sox_config_str {
		let mut file = File::create(&sox_config_cache).unwrap();
		file.write_all(current_sox_config_str.as_bytes()).unwrap();
	    } else {
		println!("--> Build files up to date; continuing");
		continue;
	    }

	    println!("--> Running output command and dumping data");
	    println!("---> {}", &config.track.output_command);

	    let output = Command::new("sh")
		.arg("-c")
		.arg(&(config.track.output_command.clone()
		       + " | head --bytes="
		       + &config.track.output_buffer))
		.output()
		.expect("Output command failed").stdout;
	    
	    let mut intermediate_name: OsString = "target/tracks/".to_owned().into();
	    intermediate_name.push(&track_name);
	    intermediate_name.push(OsStr::new("/intermediate.raw"));
	    
	    let mut file = File::create(&intermediate_name).unwrap();
	    file.write_all(&output).unwrap();

	    let mut final_name: OsString = "target/tracks/".to_owned().into();
	    final_name.push(track_name);
	    final_name.push(OsStr::new("/final.flac"));

	    // finish
	    
	    println!("--> Piping through sox");
	    
	    let mut sox_cmd = Command::new("sox");
	    sox_cmd.args(&["-b", config.sox.bit_depth.to_string().as_str()])
		.args(&["-r", config.sox.sample_rate.to_string().as_str()])
		.args(&["-c", config.sox.channels.to_string().as_str()])
		.args(&["-e", config.sox.encoding.as_str()]);
	    if let Some(other_args) = config.sox.other_options {
		for other_arg in other_args {
		    sox_cmd.arg(other_arg);
		}
	    }
	    sox_cmd.args(&["-t", "raw"])
		.arg(intermediate_name)
		.args(&["-t", "flac"])
		.arg(final_name);
	    
	    println!("---> sox {}", sox_cmd.get_args().into_iter().enumerate()
		     .map(|(i, arg)|
			  if i == 10 || i == 13 {
			      "'".to_owned()
				  + &format!("{}", String::from_utf8_lossy(arg.as_bytes()))
				  .replace("'", "\\'")
				  + "'"
			  } else {
			      format!("{}", String::from_utf8_lossy(arg.as_bytes()))
			  })
		     .collect::<Vec<String>>().join(" "));

	    let sox_output = sox_cmd.output()
		.expect("Sox command failed");

	    if !sox_output.status.success() {
		eprintln!("{}", String::from_utf8_lossy(&sox_output.stderr));
		panic!("Sox command failed");
	    }
	    
	    println!("--> Finished processing track '{}'", config.track.name);
	}

    println!("Done");
}
