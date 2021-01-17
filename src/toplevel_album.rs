use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::album_data::AlbumData;
use crate::album_name::AlbumName;

pub fn process_albums(matches: &clap::ArgMatches) {
    for album_name in get_albums(matches) {
        build_album(album_name, matches);
    }
}

pub fn clean_arg(matches: &clap::ArgMatches) {
    let album_name = AlbumName::new_from_arg(matches);
    println!("Cleaning cache for album {}", album_name.get_name());
    std::fs::remove_dir_all(album_name.dest_dir()).ok(); // empty cache
}

pub fn export_arg(matches: &clap::ArgMatches) {
    println!("Exporting...");
    let album_name = AlbumName::new_from_arg(matches);
    let album_data = AlbumData::load_from_track(&album_name);

    let old_dir = album_name.dest_dir();
    let new_dir = PathBuf::from(matches.value_of("output_dir").unwrap())
        .join(album_data.album_config.album.title);

    Command::new("rm")
        .arg("-rf")
        .arg(&new_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("export command failed");

    assert!(
        Command::new("cp")
            .arg("-r")
            .arg(old_dir)
            .arg(new_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("export command failed")
            .status
            .success(),
        "export command failed"
    );
}

pub fn build_arg(matches: &clap::ArgMatches) {
    build_album(AlbumName::new_from_arg(matches), matches);
}

pub fn play_arg(matches: &clap::ArgMatches) {
    println!("Playing via mpv...");
    let album_name = AlbumName::new_from_arg(matches);
    let album_data = AlbumData::load_from_track(&album_name);
    assert!(
        Command::new("mpv")
            .arg(
                album_name
                    .dest_dir()
                    .join(format!("{}.flac", album_data.album_config.album.title))
            )
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("mpv command failed")
            .status
            .success(),
        "mpv command failed"
    );
}

pub fn get_albums(matches: &clap::ArgMatches) -> Vec<AlbumName> {
    fs::read_dir(
        matches
            .value_of("album_dir")
            .map(|dirname| PathBuf::from(dirname))
            .unwrap_or(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("albums")),
    )
    .unwrap()
    .map(|res| AlbumName::new(res.map(|e| e.path()).unwrap().file_stem().unwrap(), matches))
    .collect()
}

pub fn build_album(album_name: AlbumName, matches: &clap::ArgMatches) {
    println!("> Building album {}", album_name);
    println!("> Loading config file");
    let config = AlbumData::load_from_track(&album_name);

    config.compile(&matches);
}
