use serde::{Deserialize, de::DeserializeOwned, Serialize};
use std::process::{Command, Stdio};
use std::fs::{self, File, metadata};
use std::path::{Path, PathBuf};
use std::io::Write;
use std::time::SystemTime;
use curl::easy::Easy;
use chrono::naive::{NaiveDateTime, NaiveTime};

use crate::TrackName;

type Clips = Vec<Clip>;
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct ClipsOpt { // serde crap
    clip: Option<Clips>,
}

pub trait Cache {
    fn load_from_cache(track_name: &TrackName) -> Option<Self>
    where Self: Sized + From<TrackConfig> + DeserializeOwned + std::fmt::Debug {
	let filename = track_name.dest_dir()
	    .join(format!("{}.toml", Self::self_type()));
	fs::read_to_string(&filename).ok()
	    .and_then(|cfg_str| {
		toml::from_str::<Self>(&cfg_str).ok()
	    })
    }
    fn write_cache(&self, track_name: &TrackName) where Self: Serialize {
	let current_sox_config_str = format!(
	    "{}",
	    toml::to_string(&self).unwrap()
		.replace("[[]]", &format!("[[{}]]", Self::self_type()))); // for vector types
	
	let mut file = File::create(
	    track_name.dest_dir().join(format!("{}.toml", Self::self_type()))
	).unwrap();
	file.write_all(current_sox_config_str.as_bytes()).unwrap();
    }
    fn self_type() -> &'static str;
}


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
    clip: Option<Clips>,
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
	let track_config: TrackConfig = toml::from_str(
	    &fs::read_to_string(track_name.source_dir().join("config.toml")).unwrap()
	).unwrap();
	
	let needs_build_update = match (track_config.output.cache, &track_config.build) {
	    (Some(false), _) => true, // will propogate
	    (_, Some(bref)) => Build::load_from_cache(&track_name)
		!= Some(bref.clone()),
	    _ => false,
	};
	
	let needs_raw_update = match needs_build_update {
	    true => true,
	    false => (Output::load_from_cache(&track_name)
		!= Some(track_config.clone().into()))
		|| !track_name.dest_dir().join(TrackData::raw_filename()).exists(),
	};
	    
	let needs_preprocessed_update = match needs_raw_update {
	    true => true,
	    false => (Sox::load_from_cache(&track_name)
		      != Some(track_config.clone().into()))
		|| !track_name.dest_dir().join(TrackData::unprocessed_filename()).exists(),
	};

	let needs_ffmpeg_update = match needs_preprocessed_update {
	    true => true,
	    false => (ClipsOpt::load_from_cache(&track_name)
		      != Some(track_config.clone().into()))
		|| !track_name.dest_dir().join(TrackData::processed_filename()).exists(),
	};
	
	Self {
	    track_config,
	    updates: Updates {
		needs_raw_update,
		needs_preprocessed_update,
		needs_build_update,
		needs_ffmpeg_update,
	    }
	}
    }
    pub fn dump_raw(&self, track_name: &TrackName) {
	let intermed_file = track_name.dest_dir().join(TrackData::raw_filename());

	std::fs::remove_file(&intermed_file).ok(); // makes cache happy
	
	let mut cmd = Command::new("sh");
	cmd.arg("-c")
	    .arg(&(self.track_config.output.output_command.clone()
		   + " | head --bytes="
		   + &self.track_config.output.output_buffer));

	if self.output().debug == Some(true) {
	    cmd.stdout(Stdio::inherit())
		.stderr(Stdio::inherit());
	}
	if self.build().is_some() {
	    cmd.current_dir(track_name.dest_dir().join("build"));
	}

	let output = cmd.output()
	    .expect("Output command failed");
	
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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Sox {
    pub bit_depth: u32,
    pub sample_rate: u32,
    pub channels: u32,
    pub encoding: String,
    pub other_options: Option<String>,
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


#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Build {
    pub build_command: String,
    pub http_sources: Vec<String>,
    pub git_sources: Vec<String>,
    pub always_rebuild: Option<bool>,
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
    }
    pub fn run(&self, track_name: &TrackName) -> bool { // is out of date
	if !self.always_rebuild.unwrap_or(false) && Self::build_lock_file(track_name).exists() {
	    println!("--> Build up to date");
	    return false;
	}
	println!("---> {}", self.build_command);
	assert!(Command::new("sh")
		.arg("-c")
		.arg(&self.build_command)
		.current_dir(track_name.dest_dir().join("build"))
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.output()
		.expect("Build command failed").status.success(),
		"Build command failed");
	File::create(Self::build_lock_file(track_name)).unwrap();
	true
    }
    fn get_lastmod_upstream(&self, source: &String) -> Option<NaiveDateTime> {
	let mut easy = Easy::new();
	easy.url(source).unwrap();
	let mut last_modified_upstream = None;
	{
	    let mut transfer = easy.transfer();
	    transfer.header_function(|data| {
		let head = String::from_utf8_lossy(data);
		if head.starts_with("Last-Modified") {
		    if let Ok(date) = NaiveDateTime::parse_from_str(
			&head.trim().chars().skip("Last-Modified: ".len()).collect::<String>(),
			"%a, %d %b %Y %T GMT") {
			last_modified_upstream = Some(date);
		    }
		}
		head.trim().len() > 0
	    }).unwrap();
	    transfer.perform().ok(); // throw away the error, we expect one from quitting early
	}
	last_modified_upstream
    }
    fn get_lastmod_downstream(&self, track_name: &TrackName, source: &String) -> Option<NaiveDateTime> {
	metadata(track_name.dest_dir().join("http")
		 .join(Path::new(source)
		       .file_name().unwrap())).ok()
	    .and_then(|m| m.modified().ok()
		      .map(|d| NaiveDateTime::from_timestamp
			   (d.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64, 0)))
    }
    pub fn http(&self, track_name: &TrackName, cache: bool) -> bool { // returns OutOfDate
	let mut out_of_date = false;
	for source in &self.http_sources {
	    let last_modified_downstream = if !cache { None } else {
		self.get_lastmod_downstream(track_name, source)
	    };
	    let last_modified_upstream = match last_modified_downstream {
		None => None,
		_ => self.get_lastmod_upstream(source),
	    };

	    match (last_modified_downstream, last_modified_upstream) {
		(Some(down), Some(up)) if up < down => continue,
		_ => (),
	    }

	    out_of_date = true;

	    println!("---> {}", source);
	    
	    assert!(Command::new("curl") // Yeah, I'm using curl(1) and libcurl.
		    .arg(source)         // If you hate it so much fix it yourself and PR.
		    .arg("-O")
		    .current_dir(track_name.dest_dir().join("http"))
		    .stdout(Stdio::inherit())
		    .stderr(Stdio::inherit())
		    .output()
		    .expect("Curl http request failed. Aborting.").status.success(),
		    "Curl http request failed. Aborting.");
	    let dl_name = Path::new(source).file_name().unwrap();
	    assert!(Command::new("cp")
		    .arg(track_name.dest_dir().join("http").join(&dl_name))
		    .arg(track_name.dest_dir().join("build").join(&dl_name))
		    .stdout(Stdio::inherit())
		    .stderr(Stdio::inherit())
		    .output()
		    .expect("cp failed. Aborting.").status.success(),
		    "cp failed. Aborting.");
	}
	out_of_date
    }
    pub fn git(&self, track_name: &TrackName) -> bool {
	let mut out_of_date = false;
	for source in &self.git_sources {
	    let git_dir = track_name.dest_dir().join("build")
		.join(Path::new(source)
		      .file_stem().unwrap());
	    
	    if !git_dir.exists() {
		assert!(Command::new("git")
		    .arg("clone")
		    .arg(source)
		    .current_dir(track_name.dest_dir().join("build"))
		    .stdout(Stdio::inherit())
		    .stderr(Stdio::inherit())
		    .output()
		    .expect("Git clone failed").status.success(),
			"Git clone failed");
		out_of_date = true;
	    } else {
		assert!(Command::new("git")
		    .arg("remote")
		    .arg("update")
		    .current_dir(&git_dir)
		    .stdout(Stdio::inherit())
		    .stderr(Stdio::inherit())
		    .output()
		    .expect("Git remote failed").status.success(),
			"Git remote failed");
		let git_status = Command::new("git")
		    .arg("status")
		    .arg("-uno")
		    .current_dir(&git_dir)
		    .output()
		    .expect("Git status failed");
		assert!(git_status.status.success(),
			"Git status failed");
		if String::from_utf8_lossy(&git_status.stdout)
		    .lines().nth(1).unwrap_or("").starts_with("Your branch is behind") {
			out_of_date = true;
			assert!(Command::new("git")
				.arg("reset")
				.arg("--hard")
				.current_dir(&git_dir)
				.stdout(Stdio::inherit())
				.stderr(Stdio::inherit())
				.output()
				.expect("Git reset failed").status.success(),
				"Git reset failed");
			assert!(Command::new("git")
				.arg("pull")
				.current_dir(&git_dir)
				.stdout(Stdio::inherit())
				.stderr(Stdio::inherit())
				.output()
				.expect("Git pull failed").status.success(),
				"Git pull failed");
		    }
	    }
	}
	out_of_date
    }
}




#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Clip {
    start: NaiveTime,
    end: NaiveTime,
    position: String,
}
