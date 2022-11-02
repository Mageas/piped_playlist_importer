# Piped Playlist Importer

Import your playlists to Piped from text files.

## **How to use**

``` text
Usage: piped_playlist_importer [OPTIONS] --authorization <AUTHORIZATION>

Options:
  -p, --playlists-directory <PLAYLISTS_DIRECTORY>
          Path of the directory containing playlists
  -a, --authorization <AUTHORIZATION>
          Authorization code found in the local storage of your browser (authToken...)
  -i, --instance <INSTANCE>
          Authentication instance [default: https://pipedapi.kavin.rocks]
      --append
          Append videos to existing playlists
  -h, --help
          Print help information
  -V, --version
          Print version information
```

## **Install instructions**

Clone the repository:
```
git clone https://gitea.heartnerds.org/Mageas/piped_playlist_importer
```

Move into the project directory:
```
cd piped_playlist_importer
```

Install the project with cargo:
```
cargo install --path=.
```
