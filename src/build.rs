use chrono::naive::NaiveDateTime;
use curl::easy::Easy;
use serde::{Deserialize, Serialize};
use std::fs::{self, metadata, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::cache::Cache;
use crate::config::TrackConfig;
use crate::track_name::TrackName;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Build {
    pub build_command: String,
    pub http_sources: Vec<String>,
    pub git_sources: Vec<String>,
    pub git_update: Option<bool>,
    pub always_rebuild: Option<bool>,
    pub copy_me: bool,
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
    fn build_lock_file(track_name: &TrackName) -> PathBuf {
        track_name.dest_dir().join("build_complete.unlock")
    }
    pub fn wipe_build_progress(&self, track_name: &TrackName) {
        std::fs::remove_file(Self::build_lock_file(track_name)).ok();
    }
    pub fn create_dirs(&self, track_name: &TrackName) {
        fs::create_dir_all(track_name.dest_dir().join("build").into_os_string()).unwrap();
        fs::create_dir_all(track_name.dest_dir().join("http").into_os_string()).unwrap();
        fs::create_dir_all(track_name.dest_dir().join("local").into_os_string()).unwrap();
    }
    pub fn run(&self, track_name: &TrackName) -> bool {
        // is out of date
        if !self.always_rebuild.unwrap_or(false) && Self::build_lock_file(track_name).exists() {
            println!("--> Build up to date");
            return false;
        }
        println!("--> Building");
        println!("---> {}", self.build_command);
        assert!(
            Command::new("sh")
                .arg("-c")
                .arg(&self.build_command)
                .current_dir(track_name.dest_dir().join("build"))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .expect("Build command failed")
                .status
                .success(),
            "Build command failed"
        );
        File::create(Self::build_lock_file(track_name)).unwrap();
        true
    }
    fn get_lastmod_upstream(&self, source: &String) -> Option<NaiveDateTime> {
        let mut easy = Easy::new();
        easy.url(source).unwrap();
        let mut last_modified_upstream = None;
        {
            let mut transfer = easy.transfer();
            transfer
                .header_function(|data| {
                    let head = String::from_utf8_lossy(data);
                    if head.starts_with("Last-Modified") {
                        if let Ok(date) = NaiveDateTime::parse_from_str(
                            &head
                                .trim()
                                .chars()
                                .skip("Last-Modified: ".len())
                                .collect::<String>(),
                            "%a, %d %b %Y %T GMT",
                        ) {
                            last_modified_upstream = Some(date);
                        }
                    }
                    head.trim().len() > 0
                })
                .unwrap();
            transfer.perform().ok(); // throw away the error, we expect one from quitting early
        }
        last_modified_upstream
    }
    fn get_lastmod_downstream(
        &self,
        track_name: &TrackName,
        source: &String,
    ) -> Option<NaiveDateTime> {
        metadata(
            track_name
                .dest_dir()
                .join("http")
                .join(Path::new(source).file_name().unwrap()),
        )
        .ok()
        .and_then(|m| {
            m.modified().ok().map(|d| {
                NaiveDateTime::from_timestamp(
                    d.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                    0,
                )
            })
        })
    }
    fn get_lastmod_local(&self, source: &PathBuf) -> Option<NaiveDateTime> {
        metadata(source).ok().and_then(|m| {
            m.modified().ok().map(|d| {
                NaiveDateTime::from_timestamp(
                    d.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                    0,
                )
            })
        })
    }
    pub fn http(&self, track_name: &TrackName, cache: bool) -> bool {
        // returns OutOfDate
        let mut out_of_date = false;
        for source in &self.http_sources {
            let last_modified_downstream = if !cache {
                None
            } else {
                self.get_lastmod_downstream(track_name, source)
            };
            let last_modified_upstream = match last_modified_downstream {
                None => None,
                _ => self.get_lastmod_upstream(source),
            };

            match (last_modified_downstream, last_modified_upstream) {
                (Some(down), Some(up)) if up < down => continue,
                (Some(_), None) => continue, // if idk upstream, assume up to date
                _ => {}
            }

            out_of_date = true;

            println!("---> {}", source);

            assert!(
                Command::new("curl") // Yeah, I'm using curl(1) and libcurl.
                    .arg(source) // If you hate it so much fix it yourself and PR.
                    .arg("-O")
                    .current_dir(track_name.dest_dir().join("http"))
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("Curl http request failed. Aborting.")
                    .status
                    .success(),
                "Curl http request failed. Aborting."
            );
            let dl_name = Path::new(source).file_name().unwrap();
            assert!(
                Command::new("cp")
                    .arg(track_name.dest_dir().join("http").join(&dl_name))
                    .arg(track_name.dest_dir().join("build").join(&dl_name))
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("cp failed. Aborting.")
                    .status
                    .success(),
                "cp failed. Aborting."
            );
        }
        out_of_date
    }
    pub fn git(&self, track_name: &TrackName) -> bool {
        let mut out_of_date = false;
        for source in &self.git_sources {
            let git_dir = track_name
                .dest_dir()
                .join("build")
                .join(Path::new(source).file_stem().unwrap());

            if !git_dir.exists() {
                assert!(
                    Command::new("git")
                        .arg("clone")
                        .arg(source)
                        .current_dir(track_name.dest_dir().join("build"))
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .output()
                        .expect("Git clone failed")
                        .status
                        .success(),
                    "Git clone failed"
                );
                out_of_date = true;
            } else {
                if self.git_update != Some(false) {
                    assert!(
                        Command::new("git")
                            .arg("remote")
                            .arg("update")
                            .current_dir(&git_dir)
                            .stdout(Stdio::inherit())
                            .stderr(Stdio::inherit())
                            .output()
                            .expect("Git remote failed")
                            .status
                            .success(),
                        "Git remote failed"
                    );
                    let git_status = Command::new("git")
                        .arg("status")
                        .arg("-uno")
                        .current_dir(&git_dir)
                        .output()
                        .expect("Git status failed");
                    assert!(git_status.status.success(), "Git status failed");
                    if String::from_utf8_lossy(&git_status.stdout)
                        .lines()
                        .nth(1)
                        .unwrap_or("")
                        .starts_with("Your branch is behind")
                    {
                        out_of_date = true;
                        assert!(
                            Command::new("git")
                                .arg("reset")
                                .arg("--hard")
                                .current_dir(&git_dir)
                                .stdout(Stdio::inherit())
                                .stderr(Stdio::inherit())
                                .output()
                                .expect("Git reset failed")
                                .status
                                .success(),
                            "Git reset failed"
                        );
                        assert!(
                            Command::new("git")
                                .arg("pull")
                                .current_dir(&git_dir)
                                .stdout(Stdio::inherit())
                                .stderr(Stdio::inherit())
                                .output()
                                .expect("Git pull failed")
                                .status
                                .success(),
                            "Git pull failed"
                        );
                    }
                }
            }
        }
        out_of_date
    }
    pub fn local(&self, track_name: &TrackName, cache: bool) -> bool {
        let mut out_of_date = false;
        for entry in WalkDir::new(track_name.source_dir()).into_iter().skip(1) {
            let e = entry.unwrap();
            let srcpath = e.path();
            let srcpath_string = srcpath
                .display()
                .to_string()
                .chars()
                .skip(format!("tracks/{}/", track_name.get_name()).len()) // Path::pop would be better but ehh
                .collect::<String>();
            let dstpath = track_name.dest_dir().join("local").join(&srcpath_string);

            let last_modified_src = if !cache {
                None
            } else {
                self.get_lastmod_local(&srcpath.to_path_buf())
            };
            let last_modified_dst = if !cache {
                None
            } else {
                self.get_lastmod_local(&dstpath)
            };

            match (last_modified_src, last_modified_dst) {
                (Some(src), Some(dst)) if src < dst => continue,
                _ => {}
            }

            println!("---> {}", srcpath_string);

            out_of_date = true;
            if srcpath.is_dir() {
                std::fs::create_dir(dstpath).expect("local mkdir failed");
            } else {
                std::fs::copy(&srcpath, dstpath).expect("local copy failed");
            }
        }
        assert!(
            Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "cp -r {}/* {}/",
                    track_name
                        .dest_dir()
                        .join("local")
                        .into_os_string()
                        .to_string_lossy(),
                    track_name
                        .dest_dir()
                        .join("build")
                        .into_os_string()
                        .to_string_lossy()
                ))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .expect("Git reset failed")
                .status
                .success(),
            "Git reset failed"
        );
        out_of_date
    }
}
