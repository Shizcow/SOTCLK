use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

#[derive(Debug)]
pub struct TrackName {
    name: OsString,
    root_dir: PathBuf,
}

impl TrackName {
    pub fn new_from_arg(matches: &clap::ArgMatches) -> Self {
        crate::toplevel::get_tracks(matches)
            .into_iter()
            .find(|tn| tn.get_name() == matches.value_of("track").unwrap())
            .expect(&format!(
                "Track '{}' not found in tracks/ directory",
                matches.value_of("track").unwrap()
            ))
    }
    pub fn new(name: &OsStr, matches: &clap::ArgMatches) -> Self {
        Self {
            name: name.to_os_string(),
            root_dir: matches
                .value_of("track_dir")
                .map(|dirname| PathBuf::from(dirname))
                .unwrap_or(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tracks")),
        }
    }
    pub fn source_dir(&self) -> PathBuf {
        self.root_dir.clone().join(&self.name)
    }
    pub fn dest_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("tracks")
            .join(&self.name)
    }
    pub fn build_dir(&self) -> PathBuf {
        self.dest_dir().join("build")
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
