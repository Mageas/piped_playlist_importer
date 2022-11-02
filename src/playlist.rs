use std::fs::ReadDir;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};

use crate::error::{PipedPlaylistImporterError, PipedPlaylistImporterResult};

pub struct Playlists {}

impl Playlists {
    pub fn new(files: ReadDir, directory: &Path) -> PipedPlaylistImporterResult<Vec<Playlist>> {
        let playlists = Self::list_directory_files(files, directory)?;
        let playlists = Self::convert_to_playlist(playlists)?;
        Ok(playlists)
    }

    /// Convert to vec of playlists
    fn convert_to_playlist(playlists: Vec<PathBuf>) -> PipedPlaylistImporterResult<Vec<Playlist>> {
        let mut return_playlists: Vec<Playlist> = vec![];
        let mut return_actual_playlist: Playlist = Playlist::new("".to_string());

        for playlist in playlists {
            let path = playlist
                .to_str()
                .ok_or_else(|| PipedPlaylistImporterError::FileName(playlist.clone()))?;
            let urls = Self::read_file(path)?;

            let name = playlist
                .file_name()
                .ok_or_else(|| PipedPlaylistImporterError::FileName(playlist.clone()))?
                .to_str()
                .ok_or_else(|| PipedPlaylistImporterError::FileName(playlist.clone()))?;

            for url in urls {
                if name != return_actual_playlist.name {
                    return_playlists.push(return_actual_playlist);
                    return_actual_playlist = Playlist::new(name.to_owned())
                }
                return_actual_playlist.push(url);
            }
        }

        return_playlists.remove(0);
        Ok(return_playlists)
    }

    /// List the files from a directory
    fn list_directory_files(
        files: ReadDir,
        directory: &Path,
    ) -> PipedPlaylistImporterResult<Vec<PathBuf>> {
        let mut output = vec![];
        for file in files {
            let file = file
                .map_err(|e| {
                    PipedPlaylistImporterError::ListFiles(e, directory.to_str().unwrap().to_owned())
                })?
                .path();
            output.push(file.to_owned());
        }
        Ok(output)
    }

    /// Read the file to a vec of strings
    fn read_file(path: &str) -> PipedPlaylistImporterResult<Vec<String>> {
        let mut lines = vec![];
        for line in BufReader::new(std::fs::File::open(path)?).lines() {
            let line =
                line.map_err(|e| PipedPlaylistImporterError::ReadLines(e, path.to_owned()))?;
            lines.push(line);
        }
        Ok(lines)
    }
}

#[derive(Debug)]
pub struct Playlist {
    pub name: String,
    pub urls: Vec<String>,
}

impl Playlist {
    fn new(name: String) -> Self {
        Self { name, urls: vec![] }
    }

    fn push(&mut self, url: String) {
        self.urls.push(url);
    }
}
