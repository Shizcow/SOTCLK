use std::fs;
use std::path::PathBuf;

use crate::album_data::AlbumData;
use crate::album_name::AlbumName;

pub fn clean_arg(matches: &clap::ArgMatches) {
    let album_name = AlbumName::new_from_arg(matches);
    println!("Cleaning cache for album {}", album_name.get_name());
    std::fs::remove_dir_all(album_name.dest_dir()).ok(); // empty cache
}

pub fn build_arg(matches: &clap::ArgMatches) {
    build_album(AlbumName::new_from_arg(matches), matches);
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
