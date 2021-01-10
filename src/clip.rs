use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use chrono::naive::NaiveTime;

use crate::cache::Cache;
use crate::config::TrackConfig;
use crate::config::TrackData;
use crate::track_name::TrackName;

pub type Clips = Vec<Clip>;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ClipsOpt { // serde crap
    clip: Option<Clips>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Clip {
    start: NaiveTime,
    end: NaiveTime,
    position: String,
}


pub trait ClipProcess {
    fn process(self, track_name: &TrackName);
}

impl ClipProcess for Clips {
    fn process(mut self, track_name: &TrackName) {
	std::fs::remove_file(track_name.dest_dir().join(TrackData::processed_filename())).ok();
	if self.len() == 0 {
	    assert!(Command::new("cp")
		    .arg(track_name.dest_dir().join(TrackData::unprocessed_filename()))
		    .arg(track_name.dest_dir().join(TrackData::processed_filename()))
		    .stdout(Stdio::inherit())
		    .stderr(Stdio::inherit())
		    .output()
		    .expect("cp failed. Aborting.").status.success(),
		    "cp failed. Aborting.");
	} else {
	    // check clips for "absolute"/"relative" correctness
	    for clip in self.iter() {
		match clip.position.as_str() {
		    "absolute" | "relative" => (),
		    other => panic!("clip position '{}' is invalid. \
				     Valid options are: relative, absolute.",
				    other),
		}
	    }
	    
	    // logically, the first clip is always absolute
	    self[0].position = "absolute".to_owned();

	    // now, fold through and make everything absolute
	    for clip_n in 1..self.len() {
		if &self[clip_n].position == "relative" {
		    let offset = self[clip_n-1].end
			.signed_duration_since(NaiveTime::from_hms(0, 0, 0));
		    self[clip_n].start += offset;
		    self[clip_n].end += offset;
		    self[clip_n].position = "absolute".to_owned();
		}
	    }

	    let filter_arg = 
		format!("aselect='{}',asetpts=N/SR/TB", self.into_iter().map(|clip| {
		    format!("between(t,{},{})",
			    clip.start.signed_duration_since(NaiveTime::from_hms(0, 0, 0))
			    .num_seconds(),
			    clip.end.signed_duration_since(NaiveTime::from_hms(0, 0, 0))
			    .num_seconds())
		}).collect::<Vec<String>>().join("+"));

	    
	    assert!(Command::new("ffmpeg")
		    .arg("-i")
		    .arg(track_name.dest_dir().join(TrackData::unprocessed_filename()))
		    .arg("-af")
		    .arg(filter_arg)
		    .arg(track_name.dest_dir().join(TrackData::processed_filename()))
		    .stdout(Stdio::inherit())
		    .stderr(Stdio::inherit())
		    .output()
		    .expect("ffmpeg failed. Aborting.").status.success(),
		    "ffmpeg failed. Aborting.");
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
	Self{clip: c.clip}
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
