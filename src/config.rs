use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs::{self, File};
use std::io::Write;

use crate::TrackName;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TrackConfig {
    pub track: Track,
    pub sox: Sox,
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
	
	let mut file = File::create(track_name.raw_file()).unwrap();
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
pub struct SoxConfig {
    pub track: Track,
    pub sox: Sox,
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
pub struct Sox {
    pub bit_depth: u32,
    pub sample_rate: u32,
    pub channels: u32,
    pub encoding: String,
    pub other_options: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Track {
    pub name: String,
    pub build_command: Option<String>,
    pub output_command: String,
    pub output_buffer: String,
}
