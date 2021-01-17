use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs;
use std::ops::Deref;

use crate::album_name::AlbumName;
use crate::config::TrackData;
use crate::track_name::TrackName;

#[derive(Clone, Debug)]
pub struct AlbumData<'a> {
    pub album_config: AlbumConfig,
    album_name: &'a AlbumName,
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

        let track_datas = self.tracks().iter().map(|track| {
            let track_str: OsString = track.into();
            let track_name = TrackName::new(&track_str, matches);
            let track_data = TrackData::load_from_track(&track_name);
            crate::toplevel_track::build_track(track_name.clone());

            std::fs::copy(
                track_name.dest_dir().join(TrackData::processed_filename()),
                self.album_name
                    .dest_dir()
                    .join(Self::track_dir_name())
                    .join(track_data.output().name.clone())
                    .with_extension("flac"),
            )
            .unwrap();

            track_data
        });

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
    album: Album, // serde crap
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
