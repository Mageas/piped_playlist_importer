use serde::{Deserialize, Serialize};

use reqwest::header::{AUTHORIZATION, USER_AGENT};
use reqwest::Client;

use crate::error::{PipedPlaylistImporterError, PipedPlaylistImporterResult};

#[derive(Debug)]
pub struct PipedClient {
    pub httpclient: Client,
    pub instance: String,
    pub authorization: String,
}

const USER_AGENT_: &str = "Mozilla/5.0 (Windows NT 10.0; rv:78.0) Gecko/20100101 Firefox/78.0";

impl PipedClient {
    /// New piped client
    pub fn new<S: AsRef<str>>(instance: S, authorization: S) -> PipedClient {
        let mut url = instance.as_ref().to_owned();
        if !url.starts_with("http") {
            url = format!("https://{}", url);
        }
        if url.ends_with('/') {
            url.pop();
        }

        PipedClient {
            httpclient: Client::new(),
            instance: url,
            authorization: authorization.as_ref().to_owned(),
        }
    }

    /// Retrieve the playlists from piped
    pub async fn get_playlists(
        &self,
    ) -> PipedPlaylistImporterResult<Vec<PipedGetPlaylistResponse>> {
        let url = format!("{}/user/playlists", self.instance);
        let response = self.get(&url).await?;
        Ok(serde_json::from_str::<Vec<PipedGetPlaylistResponse>>(
            &response,
        )?)
    }

    /// Retrieve the videos from a playlist
    pub async fn get_videos(&self, playlist_id: &str) -> PipedPlaylistImporterResult<Vec<String>> {
        let url = format!("{}/playlists/{}", self.instance, playlist_id);
        let response = self.get(&url).await?;
        Ok(
            serde_json::from_str::<PipedGetPlaylistInfosResponse>(&response)?
                .related_streams
                .iter()
                .map(|s| s.url.split("?v=").nth(1).unwrap().to_owned())
                .collect::<Vec<String>>(),
        )
    }

    /// Create a playlist
    pub async fn create_playlist(
        &self,
        name: &str,
    ) -> PipedPlaylistImporterResult<PipedCreatePlaylistResponse> {
        let url = format!("{}/user/playlists/create", self.instance);
        let body = std::collections::HashMap::from([("name", name)]);
        let response = self.post(&url, &body).await?;
        Ok(serde_json::from_str::<PipedCreatePlaylistResponse>(
            &response,
        )?)
    }

    /// Add a video to a playlist
    pub async fn add_video_to_playlist(
        &self,
        video_id: &str,
        playlist_id: &str,
    ) -> PipedPlaylistImporterResult<PipedAddVideoToPlaylistResponse> {
        let url = format!("{}/user/playlists/add", self.instance);
        let body =
            std::collections::HashMap::from([("videoId", video_id), ("playlistId", playlist_id)]);
        let response = self.post(&url, &body).await?;
        Ok(serde_json::from_str::<PipedAddVideoToPlaylistResponse>(
            &response,
        )?)
    }
}

impl PipedClient {
    /// Post request
    async fn post<S>(&self, url: &str, body: &S) -> PipedPlaylistImporterResult<String>
    where
        S: Serialize,
    {
        let response = self
            .httpclient
            .post(url)
            .json(&body)
            .header(USER_AGENT, USER_AGENT_)
            .header(AUTHORIZATION, &self.authorization)
            .send()
            .await
            .map_err(|e| PipedPlaylistImporterError::ContactApi(e, url.to_owned()))?;

        if response.status() != 200 {
            let status = response.status();
            let text = response.text().await?;
            match serde_json::from_str::<PipedErrorResponse>(&text) {
                Ok(r) => return Err(PipedPlaylistImporterError::PipedError(r)),
                Err(_) => {
                    return Err(PipedPlaylistImporterError::Request(
                        status,
                        url.to_owned(),
                        text,
                    ))
                }
            }
        }

        Ok(response
            .text()
            .await
            .map_err(|e| PipedPlaylistImporterError::ContactApi(e, url.to_owned()))?)
    }

    /// Get request
    async fn get(&self, url: &str) -> PipedPlaylistImporterResult<String> {
        let response = self
            .httpclient
            .get(url)
            .header(USER_AGENT, USER_AGENT_)
            .header(AUTHORIZATION, &self.authorization)
            .send()
            .await
            .map_err(|e| PipedPlaylistImporterError::ContactApi(e, url.to_owned()))?;

        if response.status() != 200 {
            return Err(PipedPlaylistImporterError::Request(
                response.status(),
                url.to_owned(),
                response.text().await?,
            ));
        }

        Ok(response
            .text()
            .await
            .map_err(|e| PipedPlaylistImporterError::ContactApi(e, url.to_owned()))?)
    }
}

#[derive(Debug, Deserialize)]
pub struct PipedGetPlaylistResponse {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PipedGetPlaylistInfosResponse {
    related_streams: Vec<PipedGetVideoResponse>,
}

#[derive(Debug, Deserialize)]
pub struct PipedGetVideoResponse {
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipedCreatePlaylistResponse {
    pub playlist_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PipedAddVideoToPlaylistResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct PipedErrorResponse {
    pub message: String,
}

impl std::fmt::Display for PipedErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
