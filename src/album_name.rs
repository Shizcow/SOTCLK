use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

// All this is very similar to TrackName
// It should be done with composition instead but whatever

#[derive(Debug)]
pub struct AlbumName {
    name: OsString,
    root_dir: PathBuf,
}

impl AlbumName {
    pub fn new_from_arg(matches: &clap::ArgMatches) -> Self {
        crate::toplevel_album::get_albums(matches)
            .into_iter()
            .find(|tn| tn.get_name() == matches.value_of("album").unwrap())
            .expect(&format!(
                "Album '{}' not found in albums/ directory",
                matches.value_of("album").unwrap()
            ))
    }
    pub fn new(name: &OsStr, matches: &clap::ArgMatches) -> Self {
        Self {
            name: name.to_os_string(),
            root_dir: matches
                .value_of("album_dir")
                .map(|dirname| PathBuf::from(dirname))
                .unwrap_or(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("albums")),
        }
    }
    pub fn source_file(&self) -> PathBuf {
        self.root_dir
            .clone()
            .join(&self.name)
            .with_extension("toml")
    }
    pub fn dest_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("albums")
            .join(&self.name)
    }
    pub fn build_dir(&self) -> PathBuf {
        self.dest_dir().join("build")
    }
    pub fn get_name(&self) -> String {
        self.name.to_string_lossy().to_string()
    }
}

impl std::fmt::Display for AlbumName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.to_string_lossy())
    }
}

impl AsRef<OsStr> for &AlbumName {
    fn as_ref(&self) -> &OsStr {
        &self.name
    }
}
