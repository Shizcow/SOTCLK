use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::Deref;

use crate::album_name::AlbumName;

#[derive(Clone, Debug)]
pub struct AlbumData {
    pub album_config: AlbumConfig,
    /*pub updates: Updates,*/
}
impl AlbumData {
    pub fn load_from_track(album_name: &AlbumName) -> Self {
        let album_config: AlbumConfig =
            toml::from_str(&fs::read_to_string(album_name.source_file()).unwrap()).unwrap();

        Self { album_config }
    }
    pub fn tracks(&self) -> &Vec<String> {
        &self.album_config.tracks
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
