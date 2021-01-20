use std::ffi::OsString;
use std::process::Command;

use crate::config::TrackData;
use crate::track_name::TrackName;

pub struct SoxArgs {
    args: Vec<OsString>,
    buffer_args_n_pre: usize,
    buffer_args_n_post: usize,
}

impl SoxArgs {
    pub fn new(track_name: &TrackName, config: &TrackData) -> Self {
        let mut sox_args: Vec<OsString> = vec![
            "-b".into(),
            config.sox().bit_depth.to_string().into(),
            "-r".into(),
            config.sox().sample_rate.to_string().into(),
            "-c".into(),
            config.sox().channels.to_string().into(),
            "-e".into(),
            config.sox().encoding.clone().into(),
        ];
        let other_n_pre = if let Some(other_args) = &config.sox().other_options_pre {
            let bytes = Command::new("sh")
                .arg("-c")
                .arg("for arg in $*; do echo $arg; done")
                .arg("sox")
                .arg(other_args)
                .output()
                .expect("Output command failed")
                .stdout;
            let delineated =
            // virtually guarenteed to be lossless
             String::from_utf8_lossy(&bytes);
            let mut other_n_pre = 0;
            for other_arg in delineated.lines() {
                other_n_pre += 1;
                sox_args.push(other_arg.to_string().into());
            }
            other_n_pre
        } else {
            0
        };
        sox_args.append(&mut vec![
            "-t".into(),
            "raw".into(),
            track_name
                .dest_dir()
                .join(TrackData::raw_filename())
                .into_os_string(),
            "-t".into(),
            "flac".into(),
        ]);

        let other_n_post = if let Some(other_args) = &config.sox().other_options_post {
            let bytes = Command::new("sh")
                .arg("-c")
                .arg("for arg in $*; do echo $arg; done")
                .arg("sox")
                .arg(other_args)
                .output()
                .expect("Output command failed")
                .stdout;
            let delineated =
            // virtually guarenteed to be lossless
             String::from_utf8_lossy(&bytes);
            let mut other_n_post = 0;
            for other_arg in delineated.lines() {
                other_n_post += 1;
                sox_args.push(other_arg.to_string().into());
            }
            other_n_post
        } else {
            0
        };

        sox_args.push(
            track_name
                .dest_dir()
                .join(TrackData::unprocessed_filename())
                .into_os_string(),
        );

        Self {
            args: sox_args,
            buffer_args_n_pre: other_n_pre,
            buffer_args_n_post: other_n_post,
        }
    }
    pub fn execute(&self) {
        std::fs::remove_file(self.args.last().unwrap()).ok(); // makes cache happy

        let mut sox_cmd = Command::new("sox");
        sox_cmd.args(&self.args);

        println!("---> sox {}", self);

        let sox_output = sox_cmd.output().expect("Sox command failed");

        if !sox_output.status.success() {
            eprintln!("{}", String::from_utf8_lossy(&sox_output.stderr));
            panic!("Sox command failed");
        }
    }
}

impl std::fmt::Display for SoxArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    if i == 10 + self.buffer_args_n_pre
                        || i == 13 + self.buffer_args_n_pre + self.buffer_args_n_post
                    {
                        "'".to_owned()
                            + &format!("{}", arg.to_string_lossy()).replace("'", "\\'")
                            + "'"
                    } else {
                        format!("{}", arg.to_string_lossy())
                    }
                })
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
