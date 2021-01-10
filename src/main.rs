mod sox_args;
mod track_name;
mod config;
mod build;
mod cache;
mod clip;
mod toplevel;

fn main() {
    toplevel::setup_directories();
    
    println!("Building tracks...");
    for track_name in toplevel::get_tracks() {
	    toplevel::build_track(track_name);
	}

    println!("Done");
}
