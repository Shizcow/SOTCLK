use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;

use crate::config::TrackConfig;
use crate::track_name::TrackName;

pub trait Cache {
    fn load_from_cache(track_name: &TrackName) -> Option<Self>
    where
        Self: Sized + From<TrackConfig> + DeserializeOwned + std::fmt::Debug,
    {
        let filename = track_name
            .dest_dir()
            .join(format!("{}.toml", Self::self_type()));
        fs::read_to_string(&filename)
            .ok()
            .and_then(|cfg_str| toml::from_str::<Self>(&cfg_str).ok())
    }
    fn write_cache(&self, track_name: &TrackName)
    where
        Self: Serialize,
    {
        let mut current_sox_config_str = format!(
            "{}",
            toml::to_string(&self)
                .unwrap()
                .replace("[[]]", &format!("[[{}]]", Self::self_type()))
        ); // for vector types

        if current_sox_config_str.as_str() == "[]" {
            current_sox_config_str = "".to_owned();
        }

        let mut file = File::create(
            track_name
                .dest_dir()
                .join(format!("{}.toml", Self::self_type())),
        )
        .unwrap();
        file.write_all(current_sox_config_str.as_bytes()).unwrap();
    }
    fn self_type() -> &'static str;
}
