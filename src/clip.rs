use chrono::naive::NaiveTime;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

use crate::cache::Cache;
use crate::config::TrackConfig;
use crate::config::TrackData;
use crate::track_name::TrackName;

pub type Clips = Vec<Clip>;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ClipsOpt {
    // serde crap
    clip: Option<Clips>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Clip {
    start: NaiveTime,
    end: NaiveTime,
    position: String,
}

pub trait ClipProcess {
    fn process(self, track_name: &TrackName, tempo: f64);
}

impl ClipProcess for Clips {
    fn process(mut self, track_name: &TrackName, mut tempo: f64) {
        let mut tempo_modifiers = vec![];
        while tempo > 2.0 {
            tempo_modifiers.push(2.0);
            tempo /= 2.0;
        }
        while tempo < 0.5 {
            tempo_modifiers.push(0.5);
            tempo *= 2.0;
        }
        tempo_modifiers.push(tempo);

        let tempo_arg = tempo_modifiers
            .into_iter()
            .map(|a| format!("atempo={}", a))
            .collect::<Vec<String>>()
            .join(",");

        std::fs::remove_file(track_name.dest_dir().join(TrackData::processed_filename())).ok();
        if self.len() == 0 {
            // Re-encoding fixes any potential errors that sox may encounter
            // It's pretty fast for flac anyway
            // https://gist.github.com/jgehrcke/5572c50bedf998a1fae40a80afa80357#file-flac-reencode-py-L24-L29
            println!(
                "---> ffmpeg -i {} -filter:a {} {}",
                track_name
                    .dest_dir()
                    .join(TrackData::unprocessed_filename())
                    .clone()
                    .into_os_string()
                    .to_string_lossy(),
                tempo_arg,
                track_name
                    .dest_dir()
                    .join(TrackData::processed_filename())
                    .clone()
                    .into_os_string()
                    .to_string_lossy()
            );
            assert!(
                Command::new("ffmpeg")
                    .arg("-i")
                    .arg(
                        track_name
                            .dest_dir()
                            .join(TrackData::unprocessed_filename())
                    )
                    .arg("-filter:a")
                    .arg(tempo_arg)
                    .arg(track_name.dest_dir().join(TrackData::processed_filename()))
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("ffmpeg failed. Aborting.")
                    .status
                    .success(),
                "ffmpeg failed. Aborting."
            );
        } else {
            println!("--> Editing with ffmpeg");

            // check clips for "absolute"/"relative" correctness
            for clip in self.iter() {
                match clip.position.as_str() {
                    "absolute" | "relative" => {}
                    other => panic!(
                        "clip position '{}' is invalid. \
				     Valid options are: relative, absolute.",
                        other
                    ),
                }
            }

            // logically, the first clip is always absolute
            // However, if it is relative, make sure the end is adjusted
            if &self[0].position == "relative" {
                let mut first = &mut self[0];
                first.position = "absolute".to_owned();
                first.end += first
                    .start
                    .signed_duration_since(NaiveTime::from_hms(0, 0, 0));
            }

            // now, fold through and make everything absolute
            for clip_n in 1..self.len() {
                if &self[clip_n].position == "relative" {
                    let offset = self[clip_n - 1]
                        .end
                        .signed_duration_since(NaiveTime::from_hms(0, 0, 0));
                    self[clip_n].start += offset;
                    let start_time = self[clip_n]
                        .start
                        .signed_duration_since(NaiveTime::from_hms(0, 0, 0));
                    self[clip_n].end += start_time;
                    self[clip_n].position = "absolute".to_owned();
                }
            }

            let filter_arg = format!(
                "{},aselect='{}',asetpts=N/SR/TB",
                tempo_arg,
                self.into_iter()
                    .map(|clip| {
                        format!(
                            "between(t,{},{})",
                            clip.start
                                .signed_duration_since(NaiveTime::from_hms(0, 0, 0))
                                .num_seconds(),
                            clip.end
                                .signed_duration_since(NaiveTime::from_hms(0, 0, 0))
                                .num_seconds()
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("+")
            );

            println!(
                "---> ffmpeg -i {} -af {} {}",
                track_name
                    .dest_dir()
                    .join(TrackData::unprocessed_filename())
                    .clone()
                    .into_os_string()
                    .to_string_lossy(),
                filter_arg,
                track_name
                    .dest_dir()
                    .join(TrackData::processed_filename())
                    .clone()
                    .into_os_string()
                    .to_string_lossy()
            );

            assert!(
                Command::new("ffmpeg")
                    .arg("-i")
                    .arg(
                        track_name
                            .dest_dir()
                            .join(TrackData::unprocessed_filename())
                    )
                    .arg("-af")
                    .arg(filter_arg)
                    .arg(track_name.dest_dir().join(TrackData::processed_filename()))
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .output()
                    .expect("ffmpeg failed. Aborting.")
                    .status
                    .success(),
                "ffmpeg failed. Aborting."
            );
        }
    }
}

impl From<TrackConfig> for Clips {
    fn from(c: TrackConfig) -> Self {
        c.clip.unwrap_or(vec![])
    }
}

impl From<TrackConfig> for ClipsOpt {
    fn from(c: TrackConfig) -> Self {
        Self { clip: c.clip }
    }
}

impl Cache for Clips {
    fn self_type() -> &'static str {
        "clip"
    }
}

impl Cache for ClipsOpt {
    fn self_type() -> &'static str {
        "clip"
    }
}
