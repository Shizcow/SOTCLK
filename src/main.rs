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
    toplevel::process_tracks();

    println!("Done");
}
