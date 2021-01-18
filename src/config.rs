use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::process::{Command, Stdio};

use crate::build::Build;
use crate::cache::Cache;
use crate::clip::{Clips, ClipsOpt};
use crate::track_name::TrackName;

#[derive(Clone, Debug)]
pub struct TrackData {
    pub track_config: TrackConfig,
    pub updates: Updates,
}

#[derive(Clone, Debug)]
pub struct Updates {
    pub needs_raw_update: bool,
    pub needs_preprocessed_update: bool,
    pub needs_build_update: bool,
    pub needs_ffmpeg_update: bool,
}

impl Updates {
    pub fn build_updated(&mut self) {
        self.needs_build_update = true;
        self.needs_raw_update = true;
        self.needs_preprocessed_update = true;
    }
    pub fn rebuilt(&mut self) {
        self.needs_raw_update = true;
        self.needs_preprocessed_update = true;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TrackConfig {
    pub output: Output,
    pub sox: Sox,
    pub build: Option<Build>,
    pub clip: Option<Clips>,
}

impl TrackData {
    pub fn raw_filename() -> &'static str {
        "intermediate.raw"
    }
    pub fn unprocessed_filename() -> &'static str {
        "unprocessed.flac"
    }
    pub fn processed_filename() -> &'static str {
        "processed.flac"
    }
    pub fn load_from_track(track_name: &TrackName) -> Self {
        let filename = track_name.source_dir().join("config.toml");
        let track_config: TrackConfig = toml::from_str(
            &fs::read_to_string(filename.clone())
                .expect(&format!("could not read file {}", filename.display())),
        )
        .unwrap();

        let needs_build_update = match (track_config.output.cache, &track_config.build) {
            (Some(false), _) => true, // will propogate
            (_, Some(bref)) => Build::load_from_cache(&track_name) != Some(bref.clone()),
            _ => false,
        };

        let needs_raw_update = match needs_build_update {
            true => true,
            false => {
                (Output::load_from_cache(&track_name) != Some(track_config.clone().into()))
                    || !track_name
                        .dest_dir()
                        .join(TrackData::raw_filename())
                        .exists()
            }
        };

        let needs_preprocessed_update = match needs_raw_update {
            true => true,
            false => {
                (Sox::load_from_cache(&track_name) != Some(track_config.clone().into()))
                    || !track_name
                        .dest_dir()
                        .join(TrackData::unprocessed_filename())
                        .exists()
            }
        };

        let needs_ffmpeg_update = match needs_preprocessed_update {
            true => true,
            false => {
                (ClipsOpt::load_from_cache(&track_name) != Some(track_config.clone().into()))
                    || !track_name
                        .dest_dir()
                        .join(TrackData::processed_filename())
                        .exists()
            }
        };

        Self {
            track_config,
            updates: Updates {
                needs_raw_update,
                needs_preprocessed_update,
                needs_build_update,
                needs_ffmpeg_update,
            },
        }
    }
    pub fn dump_raw(&self, track_name: &TrackName) {
        let intermed_file = track_name.dest_dir().join(TrackData::raw_filename());

        std::fs::remove_file(&intermed_file).ok(); // makes cache happy

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(
            &(self.track_config.output.output_command.clone()
                + " | head --bytes="
                + &self.track_config.output.output_buffer),
        );

        if self.output().debug == Some(true) {
            cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        }
        if self.build().is_some() {
            cmd.current_dir(track_name.dest_dir().join("build"));
        }

        let output = cmd.output().expect("Output command failed");

        let mut file = File::create(&intermed_file).unwrap();
        file.write_all(&output.stdout).unwrap();
    }
    pub fn output(&self) -> &Output {
        &self.track_config.output
    }
    pub fn sox(&self) -> &Sox {
        &self.track_config.sox
    }
    pub fn build(&self) -> &Option<Build> {
        &self.track_config.build
    }
    pub fn clips(&mut self) -> Clips {
        self.track_config.clip.clone().unwrap_or(vec![])
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Sox {
    pub bit_depth: u32,
    pub sample_rate: u32,
    pub channels: u32,
    pub encoding: String,
    pub other_options: Option<String>,
    pub tempo: Option<f64>,
}

impl Cache for Sox {
    fn self_type() -> &'static str {
        "sox"
    }
}

impl From<TrackConfig> for Sox {
    fn from(c: TrackConfig) -> Self {
        c.sox
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
    pub name: String,
    pub build_command: Option<String>,
    pub cache: Option<bool>,
    pub debug: Option<bool>,
    pub output_command: String,
    pub output_buffer: String,
}

impl Cache for Output {
    fn self_type() -> &'static str {
        "output"
    }
}

impl From<TrackConfig> for Output {
    fn from(c: TrackConfig) -> Self {
        c.output
    }
}
