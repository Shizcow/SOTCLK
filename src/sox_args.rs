use std::process::Command;
use std::ffi::OsString;

use crate::{TrackName, TrackData};

pub struct SoxArgs {
    args: Vec<OsString>,
    buffer_args_n: usize,
}

impl SoxArgs {
    pub fn new(track_name: &TrackName, config: &TrackData) -> Self {
	let mut sox_args: Vec<OsString> = vec!["-b".into(), config.sox().bit_depth.to_string().into(),
					       "-r".into(), config.sox().sample_rate.to_string().into(),
					       "-c".into(), config.sox().channels.to_string().into(),
					       "-e".into(), config.sox().encoding.clone().into()];
	let other_n = if let Some(other_args) = &config.sox().other_options {
	    let bytes = Command::new("sh")
		.arg("-c")
		.arg("for arg in $*; do echo $arg; done")
		.arg("sox")
		.arg(other_args)
		.output()
		.expect("Output command failed").stdout;
	    let delineated = // virtually guarenteed to be lossless
		String::from_utf8_lossy(&bytes);
	    let mut other_n = 0;
	    for other_arg in delineated.lines() {
		other_n += 1;
		sox_args.push(other_arg.to_string().into());
	    }
	    other_n
	} else {
	    0
	};
	sox_args.append(&mut vec!["-t".into(), "raw".into(),
				  track_name.dest_dir().join("intermediate.raw").into_os_string(),
				  "-t".into(), "flac".into(),
				  track_name.dest_dir().join("unprocessed.flac").into_os_string()]);

	Self {
	    args: sox_args,
	    buffer_args_n: other_n,
	}
    }
    pub fn execute(&self) {
	let mut sox_cmd = Command::new("sox");
	sox_cmd.args(&self.args);

	println!("---> sox {}", self);

	let sox_output = sox_cmd.output()
	    .expect("Sox command failed");

	if !sox_output.status.success() {
	    eprintln!("{}", String::from_utf8_lossy(&sox_output.stderr));
	    panic!("Sox command failed");
	}
    }
}

impl std::fmt::Display for SoxArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	write!(f, "{}", self.args.iter().enumerate()
	       .map(|(i, arg)|
		    if i == 10+self.buffer_args_n || i == 13+self.buffer_args_n {
			"'".to_owned()
			    + &format!("{}", arg.to_string_lossy())
			    .replace("'", "\\'")
			    + "'"
		    } else {
			format!("{}", arg.to_string_lossy())
		    })
	       .collect::<Vec<String>>().join(" "))
    }
}
