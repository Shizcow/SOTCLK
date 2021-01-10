use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

pub struct TrackName {
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
    pub fn raw_file(&self) -> PathBuf {
	let mut pb = PathBuf::new();
	pb.push("target");
	pb.push("tracks");
	pb.push(&self.name);
	pb.push("intermediate.raw");
	pb
    }
    pub fn unprocessed_file(&self) -> PathBuf {
	let mut pb = PathBuf::new();
	pb.push("target");
	pb.push("tracks");
	pb.push(&self.name);
	pb.push("unprocessed.flac");
	pb
    }
    pub fn config_cache(&self, cfg_name: &str) -> PathBuf {
	let mut pb = PathBuf::new();
	pb.push("target");
	pb.push("tracks");
	pb.push(&self.name);
	pb.push(cfg_name);
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
