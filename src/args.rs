use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Path of the directory containing playlists
    #[arg(short, long)]
    pub playlists_directory: Option<String>,
    /// Authorization code found in the local storage of your browser (authToken...)
    #[arg(short, long)]
    pub authorization: String,
    /// Authentication instance
    #[arg(short, long, default_value = "https://pipedapi.kavin.rocks")]
    pub instance: String,
}
