use anyhow::{Context, Result};

use clap::Parser;

mod error;

mod piped;
use piped::*;

mod playlist;
use playlist::*;

mod args;
use args::*;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let current_directory =
        std::env::current_dir().context("Unable to get the current directory from the env")?;

    let directory = match &args.playlists_directory {
        Some(input) => std::path::PathBuf::from(input),
        None => current_directory,
    };

    let directory_files = directory.read_dir().context(format!(
        "Unable to read the directory {}",
        &directory.display()
    ))?;

    let playlists = Playlists::new(directory_files, &directory)?;

    let client = PipedClient::new(&args.instance, &args.authorization);

    let piped_playlists = client.get_playlists().await?;

    let mut global_count = 0;
    playlists
        .iter()
        .for_each(|playlist| global_count += playlist.urls.len());

    let mut errors: Vec<(String, String)> = vec![];

    let mut count = 0;
    for playlist in playlists {
        if piped_playlists.iter().any(|c| c.name == playlist.name) {
            println!("Skip {}", playlist.name);
            continue;
        }

        let response = client.create_playlist(&playlist.name).await?;
        let playlist_id = &response.playlist_id;
        println!("Created {} ({})", playlist.name, playlist_id);

        for video in playlist.urls {
            let video_id = video.split('=').nth(1).unwrap();

            let response = client.add_video_to_playlist(video_id, playlist_id).await?;

            count += 1;
            if response.message == "ok" {
                println!(
                    "({}/{}) Added {} to {} ({})",
                    count, global_count, video_id, playlist.name, playlist_id
                );
            } else {
                println!(
                    "({}/{}) Error while adding {} to {} ({})",
                    count, global_count, video_id, playlist.name, playlist_id
                );
                errors.push((playlist_id.to_owned(), video_id.to_owned()));
            }
        }
    }

    for error in errors {
        println!("Unable to add {} in {}", error.0, error.1);
    }

    Ok(())
}
