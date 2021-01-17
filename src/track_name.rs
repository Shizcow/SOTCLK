use chrono::NaiveTime;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct TrackName {
    name: OsString,
    root_dir: PathBuf,
}

impl TrackName {
    pub fn get_runtime(&self) -> NaiveTime {
        let cmd = Command::new("ffprobe")
            .arg("-v")
            .arg("error")
            .arg("-show_entries")
            .arg("format=duration")
            .arg("-of")
            .arg("default=noprint_wrappers=1:nokey=1")
            .arg("-sexagesimal")
            .arg(
                self.dest_dir()
                    .join(crate::config::TrackData::unprocessed_filename())
                    .into_os_string(),
            )
            .output()
            .expect("Build command failed");

        assert!(cmd.status.success(), "Build command failed");

        let mut time_unformatted = String::from_utf8_lossy(&cmd.stdout)
            .to_string()
            .trim()
            .to_owned();

        let mut decimal: String = time_unformatted
            .chars()
            .skip_while(|c| c != &'.')
            .take(10)
            .collect();
        while decimal.len() < 10 {
            decimal.push('0');
        }

        time_unformatted = time_unformatted
            .chars()
            .take_while(|c| c != &'.')
            .chain(decimal.chars())
            .collect();

        NaiveTime::parse_from_str(&time_unformatted, "%T.%9f").unwrap()
    }
    pub fn new_from_arg(matches: &clap::ArgMatches) -> Self {
        crate::toplevel_track::get_tracks(matches)
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
        write!(f, "{}", self.get_name())
    }
}

impl AsRef<OsStr> for &TrackName {
    fn as_ref(&self) -> &OsStr {
        &self.name
    }
}
