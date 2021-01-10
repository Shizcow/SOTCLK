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
    pub fn source_dir(&self) -> PathBuf {
	let mut pb = PathBuf::new();
	pb.push("tracks");
	pb.push(&self.name);
	pb
    }
    pub fn dest_dir(&self) -> PathBuf {
	let mut pb = PathBuf::new();
	pb.push("target");
	pb.push("tracks");
	pb.push(&self.name);
	pb
    } 
    pub fn build_dir(&self) -> PathBuf {
	let mut pb = self.dest_dir();
	pb.push("build");
	pb
    }
    pub fn get_name(&self) -> String {
	self.name.to_string_lossy().to_string()
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
