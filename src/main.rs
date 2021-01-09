use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs::{self, File};
use std::io::Write;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;


#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TrackConfig {
    track: Track,
    sox: Sox,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Sox {
    bit_depth: u32,
    sample_rate: u32,
    channels: u32,
    encoding: String,
    other_options: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Track {
    name: String,
    build_command: Option<String>,
    output_command: String,
    output_buffer: String,
}

struct SoxArgs {
    args: Vec<OsString>,
    buffer_args_n: usize,
}

fn main() {
    // create track directories
    println!("Creating build directories...");
    fs::create_dir_all("target/tracks").unwrap(); // TODO cargo root
    for dir in fs::read_dir("tracks").unwrap()
	.map(|res| res.map(|e| e.path()).unwrap()) {
	    let mut new_dir_name = OsString::new();
	    new_dir_name.push(OsStr::new("target/tracks/"));
	    new_dir_name.push(dir.file_name().unwrap());
	    fs::create_dir_all(new_dir_name).unwrap();
	}

    
    println!("Building tracks...");
    for track_name in fs::read_dir("tracks").unwrap()
	.map(|res| res.map(|e| e.path()).unwrap().file_name().unwrap().to_owned()){
	    
	    println!("-> Building track {}",
		     String::from_utf8_lossy(track_name.as_bytes()));
	    
	    println!("--> Loading config file");
	    let config = load_config(&track_name);

	    println!("--> Checking build cache");
	    if Some(&config) != load_cache(&track_name).as_ref() {
		write_cache(&track_name, &config);
	    } else {
		println!("--> Build files up to date; continuing");
		//continue;
	    }

	    println!("--> Running output command and dumping data");
	    println!("---> {}", &config.track.output_command);

	    dump_raw(&track_name, &config);

	    // finish
	    
	    println!("--> Piping through sox");
	    
	    let sox_args = create_sox_args(&track_name, &config);
	    
	    dump_sox(sox_args);
	    
	    println!("--> Finished processing track '{}'", config.track.name);
	}

    println!("Done");
}

fn log_cmd(sox_args: &SoxArgs) {
    println!("---> sox {}", sox_args.args.iter().enumerate()
	     .map(|(i, arg)|
		  if i == 10+sox_args.buffer_args_n || i == 13+sox_args.buffer_args_n {
		      "'".to_owned()
			  + &format!("{}", String::from_utf8_lossy(arg.as_bytes()))
			  .replace("'", "\\'")
			  + "'"
		  } else {
		      format!("{}", String::from_utf8_lossy(arg.as_bytes()))
		  })
	     .collect::<Vec<String>>().join(" "));
}

fn load_config(track_name: &OsString) -> TrackConfig {
    let mut config_file: OsString = "tracks/".to_owned().into();
    config_file.push(&track_name);
    config_file.push(OsStr::new("/config.toml"));
    
    toml::from_str(
	&fs::read_to_string(config_file).unwrap()
    ).unwrap()
}

fn load_cache(track_name: &OsString) -> Option<TrackConfig> {
    let mut sox_config_cache: OsString = "target/tracks/".to_owned().into();
    sox_config_cache.push(&track_name);
    sox_config_cache.push(OsStr::new("/sox.toml"));
    
    toml::from_str(
	&fs::read_to_string(sox_config_cache).unwrap()
    ).ok() // if invalid just regenerate anyway
}

fn write_cache(track_name: &OsString, config: &TrackConfig) {
    let mut sox_config_cache: OsString = "target/tracks/".to_owned().into();
    sox_config_cache.push(&track_name);
    sox_config_cache.push(OsStr::new("/sox.toml"));
    
    let current_sox_config_str = format!(
	"[track]\n{}\n[sox]\n{}",
	toml::to_string(&config.track).unwrap(),
	toml::to_string(&config.sox).unwrap());
    
    let mut file = File::create(&sox_config_cache).unwrap();
    file.write_all(current_sox_config_str.as_bytes()).unwrap();
}

fn dump_raw(track_name: &OsString, config: &TrackConfig) {
    let output = Command::new("sh")
	.arg("-c")
	.arg(&(config.track.output_command.clone()
	       + " | head --bytes="
	       + &config.track.output_buffer))
	.output()
	.expect("Output command failed").stdout;
    
    let mut intermediate_name: OsString = "target/tracks/".to_owned().into();
    intermediate_name.push(&track_name);
    intermediate_name.push(OsStr::new("/intermediate.raw"));
    
    let mut file = File::create(&intermediate_name).unwrap();
    file.write_all(&output).unwrap();
}

fn create_sox_args(track_name: &OsString, config: &TrackConfig) -> SoxArgs {
    let mut intermediate_name: OsString = "target/tracks/".to_owned().into();
    intermediate_name.push(&track_name);
    intermediate_name.push(OsStr::new("/intermediate.raw"));
    
    let mut final_name: OsString = "target/tracks/".to_owned().into();
    final_name.push(track_name);
    final_name.push(OsStr::new("/final.flac"));

    let mut sox_args: Vec<OsString> = vec!["-b".into(), config.sox.bit_depth.to_string().into(),
					   "-r".into(), config.sox.sample_rate.to_string().into(),
					   "-c".into(), config.sox.channels.to_string().into(),
					   "-e".into(), config.sox.encoding.clone().into()];
    let other_n = if let Some(other_args) = &config.sox.other_options {
	let delineated = Command::new("sh")
	    .arg("-c")
	    .arg("for arg in $*; do echo $arg; done")
	    .arg("sox")
	    .arg(other_args)
	    .output()
	    .expect("Output command failed").stdout;
	let mut other_n = 0;
	for other_arg in delineated.split(|c| *c == '\n' as u8) {
	    if other_arg.len() > 0 {
		other_n += 1;
		sox_args.push(OsStr::from_bytes(other_arg).to_os_string());
	    }
	}
	other_n
    } else {
	0
    };
    sox_args.append(&mut vec!["-t".into(), "raw".into(),
			      intermediate_name,
			      "-t".into(), "flac".into(),
			      final_name]);

    SoxArgs{
	args: sox_args,
	buffer_args_n: other_n,
    }
}

fn dump_sox(sox_args: SoxArgs) {
    let mut sox_cmd = Command::new("sox");
    sox_cmd.args(&sox_args.args);

    log_cmd(&sox_args);

    let sox_output = sox_cmd.output()
	.expect("Sox command failed");

    if !sox_output.status.success() {
	eprintln!("{}", String::from_utf8_lossy(&sox_output.stderr));
	panic!("Sox command failed");
    }
}
