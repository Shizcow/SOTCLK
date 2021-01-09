use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs::{self, File};
use std::path::PathBuf;
use std::io::Write;
use std::ffi::{OsStr, OsString};

struct TrackName {
    name: OsString
}

impl TrackName {
    pub fn new(name: &OsStr) -> Self {
	Self {
	    name: name.to_os_string(),
	}
    }
    pub fn config_file(&self) -> PathBuf {
	let mut pb = PathBuf::new();
	pb.push("tracks");
	pb.push(&self.name);
	pb.push("config.toml");
	pb
    }
    pub fn intermediate_file(&self) -> PathBuf {
	let mut pb = PathBuf::new();
	pb.push("target");
	pb.push("tracks");
	pb.push(&self.name);
	pb.push("intermediate.raw");
	pb
    }
    pub fn final_file(&self) -> PathBuf {
	let mut pb = PathBuf::new();
	pb.push("target");
	pb.push("tracks");
	pb.push(&self.name);
	pb.push("final.flac");
	pb
    }
    pub fn sox_config_cache(&self) -> PathBuf {
	let mut pb = PathBuf::new();
	pb.push("target");
	pb.push("tracks");
	pb.push(&self.name);
	pb.push("sox.toml");
	pb
    }
}

impl std::fmt::Display for TrackName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}", self.name.to_string_lossy())
    }
}

impl AsRef<OsStr> for &TrackName {
    fn as_ref(&self) -> &OsStr {
        &self.name
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct TrackConfig {
    track: Track,
    sox: Sox,
}

impl TrackConfig {
    pub fn load_from_track(track_name: &TrackName) -> Self {
	toml::from_str(
	    &fs::read_to_string(track_name.config_file()).unwrap()
	).unwrap()
    }
    pub fn dump_raw(&self, track_name: &TrackName) {
	let output = Command::new("sh")
	    .arg("-c")
	    .arg(&(self.track.output_command.clone()
		   + " | head --bytes="
		   + &self.track.output_buffer))
	    .output()
	    .expect("Output command failed").stdout;
	
	let mut file = File::create(track_name.intermediate_file()).unwrap();
	file.write_all(&output).unwrap();
    }
    pub fn write_cache(&self, track_name: &TrackName) {
	let current_sox_config_str = format!(
	    "[track]\n{}\n[sox]\n{}",
	    toml::to_string(&self.track).unwrap(),
	    toml::to_string(&self.sox).unwrap());
	
	let mut file = File::create(track_name.sox_config_cache()).unwrap();
	file.write_all(current_sox_config_str.as_bytes()).unwrap();
    }
}

#[derive(Clone, Serialize, PartialEq)]
struct SoxConfig {
    track: Track,
    sox: Sox,
}

impl SoxConfig {
    pub fn load_from_cache(track_name: &TrackName) -> Option<Self> {
	if let Ok(cfg_str) = fs::read_to_string(track_name.sox_config_cache()) {
	    toml::from_str::<TrackConfig>(&cfg_str)
		.ok().map(|o| o.into()) // if invalid just regenerate anyway
	} else {
	    None
	}
    }
}

impl Into<SoxConfig> for TrackConfig {
    fn into(self) -> SoxConfig {
	SoxConfig {
	    track: self.track,
	    sox: self.sox,
	}
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Sox {
    bit_depth: u32,
    sample_rate: u32,
    channels: u32,
    encoding: String,
    other_options: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct Track {
    name: String,
    build_command: Option<String>,
    output_command: String,
    output_buffer: String,
}

struct SoxArgs {
    args: Vec<OsString>,
    buffer_args_n: usize,
}

impl SoxArgs {
    pub fn new(track_name: &TrackName, config: &TrackConfig) -> Self {
	let mut sox_args: Vec<OsString> = vec!["-b".into(), config.sox.bit_depth.to_string().into(),
					       "-r".into(), config.sox.sample_rate.to_string().into(),
					       "-c".into(), config.sox.channels.to_string().into(),
					       "-e".into(), config.sox.encoding.clone().into()];
	let other_n = if let Some(other_args) = &config.sox.other_options {
	    let bytes = Command::new("sh")
		.arg("-c")
		.arg("for arg in $*; do echo $arg; done")
		.arg("sox")
		.arg(other_args)
		.output()
		.expect("Output command failed").stdout;
	    let delineated = // virtually guarenteed to be lossless
		String::from_utf8_lossy(&bytes);
	    let mut other_n = 0;
	    for other_arg in delineated.lines() {
		other_n += 1;
		sox_args.push(other_arg.to_string().into());
	    }
	    other_n
	} else {
	    0
	};
	sox_args.append(&mut vec!["-t".into(), "raw".into(),
				  track_name.intermediate_file().into_os_string(),
				  "-t".into(), "flac".into(),
				  track_name.final_file().into_os_string()]);

	Self {
	    args: sox_args,
	    buffer_args_n: other_n,
	}
    }
    pub fn execute(&self) {
	let mut sox_cmd = Command::new("sox");
	sox_cmd.args(&self.args);

	println!("---> sox {}", self);

	let sox_output = sox_cmd.output()
	    .expect("Sox command failed");

	if !sox_output.status.success() {
	    eprintln!("{}", String::from_utf8_lossy(&sox_output.stderr));
	    panic!("Sox command failed");
	}
    }
}

impl std::fmt::Display for SoxArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}", self.args.iter().enumerate()
	       .map(|(i, arg)|
		    if i == 10+self.buffer_args_n || i == 13+self.buffer_args_n {
			"'".to_owned()
			    + &format!("{}", arg.to_string_lossy())
			    .replace("'", "\\'")
			    + "'"
		    } else {
			format!("{}", arg.to_string_lossy())
		    })
	       .collect::<Vec<String>>().join(" "))
    }
}

fn main() {
    // create track directories
    println!("Creating build directories...");
    fs::create_dir_all("target/tracks").unwrap(); // TODO cargo root
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
	    } else {
		println!("--> Build files up to date; continuing");
		//continue;
	    }

	    println!("--> Running output command and dumping data");
	    println!("---> {}", &config.track.output_command);

	    config.dump_raw(&track_name);

	    // finish
	    
	    println!("--> Piping through sox");
	    
	    SoxArgs::new(&track_name, &config).execute();
	    
	    println!("--> Finished processing track '{}'", config.track.name);
	}

    println!("Done");
}
