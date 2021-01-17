use chrono::naive::NaiveDateTime;
use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs::{self, metadata, File};
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::SystemTime;

use crate::album_name::AlbumName;
use crate::config::TrackData;
use crate::track_name::TrackName;

#[derive(Clone, Debug)]
pub struct AlbumData<'a> {
    pub album_config: AlbumConfig,
    pub album_name: &'a AlbumName,
}
impl<'a> AlbumData<'a> {
    pub fn load_from_track(album_name: &'a AlbumName) -> Self {
        let album_config: AlbumConfig =
            toml::from_str(&fs::read_to_string(album_name.source_file()).unwrap()).unwrap();

        Self {
            album_config,
            album_name,
        }
    }
    pub fn tracks(&self) -> &Vec<String> {
        &self.album_config.tracks
    }

    pub fn compile(&self, matches: &clap::ArgMatches) {
        self.create_dirs();

        //////////////// setup and cache

        let mut out_of_date = false;
        let track_datas: Vec<TrackData> = self
            .tracks()
            .iter()
            .map(|track| {
                let track_str: OsString = track.into();
                let track_name = TrackName::new(&track_str, matches);
                let track_data = TrackData::load_from_track(&track_name);
                crate::toplevel_track::build_track(track_name.clone());

                let old_path = track_name.dest_dir().join(TrackData::processed_filename());
                let new_path = self
                    .album_name
                    .dest_dir()
                    .join(Self::track_dir_name())
                    .join(format!("{}.flac", track_data.output().name.clone()));

                // yeah it's copy and paste but whatever
                let time_old = metadata(&old_path).ok().and_then(|m| {
                    m.modified().ok().map(|d| {
                        NaiveDateTime::from_timestamp(
                            d.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                            0,
                        )
                    })
                });
                let time_new = metadata(&new_path).ok().and_then(|m| {
                    m.modified().ok().map(|d| {
                        NaiveDateTime::from_timestamp(
                            d.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                            0,
                        )
                    })
                });

                // only copy if required
                match (time_old, time_new) {
                    (Some(old), Some(new)) if new > old => (),
                    _ => {
                        out_of_date = true;
                        fs::copy(old_path, new_path).map(|_| ()).unwrap()
                    }
                }

                track_data
            })
            .collect();

        let dest_file = self
            .album_name
            .dest_dir()
            .join(format!("{}.flac", self.album_config.album.title));

        if !out_of_date && dest_file.exists() {
            println!(">Album up to date; continuing");
        } else {
            ////////////// master-cut creation

            let empty_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("silence")
                .with_extension("flac");

            assert!(
                Command::new("sox")
                    .arg("-n")
                    .arg("-r")
                    .arg("44100")
                    .arg("-b")
                    .arg("16")
                    .arg("-c")
                    .arg("2")
                    .arg("-L")
                    .arg(empty_file.clone().into_os_string())
                    .arg("trim")
                    .arg("0.0")
                    .arg("2.0")
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("Sox command failed")
                    .status
                    .success(),
                "Sox command failed"
            );

            let files: Vec<OsString> = track_datas
                .iter()
                .map(|td| {
                    self.album_name
                        .dest_dir()
                        .join(Self::track_dir_name())
                        .join(format!("{}.flac", td.output().name.clone()))
                })
                .map(|p| {
                    std::iter::once(p.into_os_string())
                        .chain(std::iter::once(empty_file.clone().into_os_string()))
                })
                .flatten()
                .collect();

            println!("Writing album full-format file");
            let fstr = files
                .iter()
                .map(|s| s.clone().into_string().unwrap())
                .map(|name| format!("file '{}'", name.replace("'", "\\'")))
                .collect::<Vec<String>>()
                .join("\n");

            let fpath = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("tracklist_internal");
            let mut ffile = File::create(fpath.clone()).unwrap();

            ffile.write_all(fstr.as_bytes()).unwrap();

            println!(
                "---> ffmpeg -f concat -safe 0 -i {} -y {:?}",
                fpath.clone().into_os_string().into_string().unwrap(),
                dest_file.clone().clone().into_os_string()
            );

            assert!(
                Command::new("ffmpeg")
                    .arg("-f")
                    .arg("concat")
                    .arg("-safe")
                    .arg("0")
                    .arg("-i")
                    .arg(fpath.into_os_string())
                    .arg("-y")
                    .arg(dest_file.into_os_string())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("Build command failed")
                    .status
                    .success(),
                "Build command failed"
            );
        }

        ////////////////// tracklist generation
        println!(
            "{:?}",
            self.tracks()
                .iter()
                .map(|track| {
                    let track_str: OsString = track.into();
                    let track_name = TrackName::new(&track_str, matches);
                    track_name.get_runtime()
                })
                .fold(vec![], |mut acc, cur| {
                    let prev_time = acc.last().unwrap_or(&Duration::zero()).clone();
                    let cur_absolute_time = cur + prev_time;
                    acc.push(cur_absolute_time);
                    acc
                })
        );

        ////////////////// cleaning up

        println!(
            "Generated album '{}' with track list:\n{}",
            self.album_config.title,
            track_datas
                .into_iter()
                .map(|t| t.output().name.clone())
                .enumerate()
                .map(|(i, s)| format!("{}: {}", i, s))
                .collect::<Vec<String>>()
                .join("\n")
        );
    }

    fn create_dirs(&self) {
        println!("\nCreating album directories...");
        fs::create_dir_all(self.album_name.dest_dir().join(Self::track_dir_name())).unwrap();
    }

    fn track_dir_name() -> &'static str {
        "individual_tracks"
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AlbumConfig {
    pub album: Album, // serde crap
}

impl Deref for AlbumConfig {
    type Target = Album;

    fn deref(&self) -> &Self::Target {
        &self.album
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Album {
    pub title: String,
    pub tracks: Vec<String>,
}
