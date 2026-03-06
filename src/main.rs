use bevy::prelude::*;
use clap::Parser;

use simai_player::AppPlugin;
use simai_player::cli::CliArgs;

fn main() {
	let args = CliArgs::parse();
    
    // Get the configuration or exit with error message
    let config = match args.get_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("\nRun with --help for usage information.");
            std::process::exit(1);
        }
    };
    
    println!("✓ Chart path: {:?}", config.chart_path);
    println!("✓ Song path: {:?}", config.song_path);
    println!("Starting player...\n");
    
    App::new()
        .insert_resource(config)
        .add_plugins(AppPlugin)
        .run();
}
