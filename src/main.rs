use anyhow::{Context, Result};

use clap::Parser;

mod error;
use error::*;

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

    let local_playlists = Playlists::new(directory_files, &directory)?;

    let client = PipedClient::new(&args.instance, &args.authorization);

    let piped_playlists = client.get_playlists().await?;

    let mut global_count = 0;
    local_playlists
        .iter()
        .for_each(|playlist| global_count += playlist.urls.len());

    let mut errors: Vec<(String, String, String)> = vec![];

    let mut count = 0;
    for playlist in local_playlists {
        let mut playlist_videos = None;
        let playlist_id = match piped_playlists.iter().find(|c| c.name == playlist.name) {
            Some(some_playlist) => {
                if args.append {
                    playlist_videos = Some(client.get_videos(&some_playlist.id).await?);
                    some_playlist.id.to_owned()
                } else {
                    println!("Skip {}", some_playlist.name);
                    count += playlist.urls.len();
                    continue;
                }
            }
            None => {
                let response = client.create_playlist(&playlist.name).await?;
                let playlist_id = &response.playlist_id;
                println!("Created {} ({})", playlist.name, playlist_id);
                playlist_id.to_owned()
            }
        };

        for video in playlist.urls {
            let video_id = video.split('=').nth(1).unwrap();

            if let Some(videos) = &playlist_videos {
                if videos.contains(&video_id.to_owned()) {
                    println!(
                        "({}/{}) Skip {} for {} ({}), already in the playlist",
                        count, global_count, video_id, playlist.name, playlist_id
                    );
                    count += 1;
                    continue;
                }
            }

            let response = match client.add_video_to_playlist(video_id, &playlist_id).await {
                Ok(res) => res,
                Err(e) => match e {
                    PipedPlaylistImporterError::PipedError(e) => {
                        eprintln!(
                            "({}/{}) Error while adding {} to {} ({}): {}",
                            count, global_count, video_id, playlist.name, playlist_id, e.message
                        );
                        errors.push((playlist_id.to_owned(), video_id.to_owned(), e.message));
                        continue;
                    }
                    _ => return Err(e.into()),
                },
            };

            count += 1;
            if response.message == "ok" {
                println!(
                    "({}/{}) Added {} to {} ({})",
                    count, global_count, video_id, playlist.name, playlist_id
                );
            } else {
                eprintln!(
                    "({}/{}) Error while adding {} to {} ({})",
                    count, global_count, video_id, playlist.name, playlist_id
                );
                errors.push((playlist_id.to_owned(), video_id.to_owned(), "".to_owned()));
            }
        }
    }

    for error in errors {
        eprintln!("Unable to add {} in ({}) : {}", error.1, error.0, error.2);
    }

    Ok(())
}
