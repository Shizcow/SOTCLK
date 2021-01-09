use serde::Deserialize;
use std::process::Command;
use std::fs::{self, File};
use std::io::Write;

#[derive(Deserialize, Debug)]
struct TrackConfig {
    track: Track,
    sox: Sox,
}

#[derive(Deserialize, Debug)]
struct Sox {
    bit_depth: u32,
    sample_rate: u32,
    channels: u32,
    encoding: String,
    other_options: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Track {
    name: String,
    build_command: Option<String>,
    output_command: String,
    output_buffer: String,
}

fn main() {
    // load a toml file
    let config: TrackConfig = toml::from_str(
	&fs::read_to_string("tracks/ls/config.toml").unwrap()
    ).unwrap();

    let output = Command::new("sh")
	.arg("-c")
	.arg(&(config.track.output_command.clone()
	       + " | head --bytes="
	       + &config.track.output_buffer))
        .output()
        .expect("Output command failed").stdout;
    
    let mut file = File::create("output.raw").unwrap();
    file.write_all(&output).unwrap();
}
