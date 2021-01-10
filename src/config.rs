use serde::{Deserialize, de::DeserializeOwned, Serialize};
use std::process::Command;
use std::fs::{self, File};
use std::io::Write;

use crate::TrackName;

pub trait Cache {
    fn load_from_cache(track_name: &TrackName) -> Option<Self>
    where Self: Sized + From<TrackConfig> + DeserializeOwned + std::fmt::Debug {
	let filename = track_name.dest_dir()
	    .join(format!("{}.toml", Self::self_type()));
	fs::read_to_string(&filename).ok()
	    .and_then(|cfg_str| {
		toml::from_str::<Self>(&cfg_str).ok()
	    })
    }
    fn write_cache(&self, track_name: &TrackName) where Self: Serialize {
	let current_sox_config_str = format!(
	    "{}",
	    toml::to_string(&self).unwrap());
	
	let mut file = File::create(
	    track_name.dest_dir().join(format!("{}.toml", Self::self_type()))
	).unwrap();
	file.write_all(current_sox_config_str.as_bytes()).unwrap();
    }
    fn self_type() -> &'static str;
}


#[derive(Clone, Debug)]
pub struct TrackData {
    pub track_config: TrackConfig,
    pub needs_raw_update: bool,
    pub needs_preprocessed_update: bool,
    pub needs_build_update: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TrackConfig {
    pub output: Output,
    pub sox: Sox,
    pub build: Option<Build>,
}

impl TrackData {
    pub fn load_from_track(track_name: &TrackName) -> Self {
	let track_config: TrackConfig = toml::from_str(
	    &fs::read_to_string(track_name.source_dir().join("config.toml")).unwrap()
	).unwrap();
	
	let needs_build_update = match (track_config.output.cache, &track_config.build) {
	    (Some(false), _) => true, // will propogate
	    (_, Some(bref)) => Build::load_from_cache(&track_name)
		!= Some(bref.clone()),
	    _ => false,
	};
	
	let needs_raw_update = match needs_build_update {
	    true => true,
	    false => Output::load_from_cache(&track_name)
		!= Some(track_config.output.clone()),
	};
	    
	let needs_preprocessed_update = match needs_raw_update {
	    true => true,
	    false => Sox::load_from_cache(&track_name)
		!= Some(track_config.sox.clone()),
	};
	
	Self {
	    track_config,
	    needs_raw_update,
	    needs_preprocessed_update,
	    needs_build_update,
	}
    }
    pub fn dump_raw(&self, track_name: &TrackName) {
	let output = Command::new("sh")
	    .arg("-c")
	    .arg(&(self.track_config.output.output_command.clone()
		   + " | head --bytes="
		   + &self.track_config.output.output_buffer))
	    .output()
	    .expect("Output command failed").stdout;
	
	let mut file = File::create(track_name.dest_dir().join("intermediate.raw")).unwrap();
	file.write_all(&output).unwrap();
    }
    pub fn track(&self) -> &Output {
	&self.track_config.output
    }
    pub fn sox(&self) -> &Sox {
	&self.track_config.sox
    }
    pub fn build(&self) -> &Option<Build> {
	&self.track_config.build
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

impl Cache for Sox {
    fn self_type() -> &'static str {
	"sox"
    }
}

impl From<TrackConfig> for Sox {
    fn from(c: TrackConfig) -> Self {
	c.sox
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
    pub name: String,
    pub build_command: Option<String>,
    pub cache: Option<bool>,
    pub output_command: String,
    pub output_buffer: String,
}

impl Cache for Output {
    fn self_type() -> &'static str {
	"output"
    }
}

impl From<TrackConfig> for Output {
    fn from(c: TrackConfig) -> Self {
	c.output
    }
}


#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Build {
    pub build_command: String,
    pub http_sources: Vec<String>,
    pub git_sources: Vec<String>,
}

impl Cache for Build {
    fn self_type() -> &'static str {
	"build"
    }
}

impl From<TrackConfig> for Build {
    fn from(c: TrackConfig) -> Self {
	c.build.unwrap() // is only called when this is true anyway
    }
}

impl Build {
    pub fn create_dir(&self, track_name: &TrackName) {
	
    }
}
