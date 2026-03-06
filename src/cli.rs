use bevy::ecs::resource::Resource;
use clap::{ArgGroup, Parser};
use std::path::PathBuf;

/// You must specify EITHER a folder OR both chart and song files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(&["folder", "chart"])
))]
#[command(after_help = "EXAMPLES:\n  \
    simai-player --folder \"songs/My Song\"\n  \
    simai-player -f \"songs/Cool Track\" --chart-name expert.txt\n  \
    simai-player --chart charts/master.txt --song audio/track.ogg\n  \
    simai-player -c chart.txt -s song.wav")]
pub struct CliArgs {
    /// Path to the song folder containing maidata.txt and audio files
    #[arg(short, long, value_name = "FOLDER")]
    pub folder: Option<PathBuf>,
    
    /// Path to the chart file (e.g., maidata.txt)
    /// (requires --song to be specified)
    #[arg(short, long, value_name = "CHART_FILE", requires = "song")]
    pub chart: Option<PathBuf>,
    
    /// Path to the song audio file
    /// (requires --chart to be specified)
    #[arg(short, long, value_name = "SONG_FILE", requires = "chart", default_value = "track")]
    pub song: Option<PathBuf>,
    
    /// Chart file name when using --folder mode (default: maidata.txt)
    #[arg(long, value_name = "NAME", default_value = "maidata.txt", requires = "folder")]
    pub chart_name: String,
    
    #[arg(long, value_name = "OFFSET", default_value = "0")]
    pub offset: i64,
}

#[derive(Debug, Clone, Resource)]
pub struct ChartConfig {
    pub chart_path: PathBuf,
    pub song_path: PathBuf,
    pub offset: i64,
}

impl CliArgs {
    pub fn get_config(&self) -> Result<ChartConfig, String> {
        // Mode 1: Folder specified
        if let Some(folder) = &self.folder {
            let chart_path = folder.join(&self.chart_name);
            
            // Validate chart file exists
            if !chart_path.exists() {
                return Err(format!("Chart file not found: {:?}", chart_path));
            }
            
            // Try to find audio file in the folder
            let song_path = self.find_audio_file(folder)?;
            
            return Ok(ChartConfig {
                chart_path,
                song_path,
                offset: self.offset,
            });
        }
        
        // Mode 2: Individual files specified
        if let (Some(chart), Some(song)) = (&self.chart, &self.song) {
            // Validate files exist
            if !chart.exists() {
                return Err(format!("Chart file not found: {:?}", chart));
            }
            if !song.exists() {
                return Err(format!("Song file not found: {:?}", song));
            }
            
            return Ok(ChartConfig {
                chart_path: chart.clone(),
                song_path: song.clone(),
                offset: self.offset,
            });
        }
        
        // This should never happen due to clap's validation
        unreachable!("clap should have validated input arguments")
    }
    
    /// Find the first audio file in a folder (supports .wav, .ogg, .mp3)
    fn find_audio_file(&self, folder: &PathBuf) -> Result<PathBuf, String> {
        let audio_extensions = ["wav", "ogg", "mp3", "flac"];
        
        if !folder.exists() {
            return Err(format!("Folder does not exist: {:?}", folder));
        }
        
        let entries = std::fs::read_dir(folder)
            .map_err(|e| format!("Failed to read directory {:?}: {}", folder, e))?;
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if audio_extensions.contains(&ext.to_str().unwrap_or("")) {
                        return Ok(path);
                    }
                }
            }
        }
        
        Err(format!("No audio file (.wav, .ogg, .mp3, .flac) found in folder: {:?}", folder))
    }
}
