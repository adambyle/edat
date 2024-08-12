use std::{fs, ops::Deref};

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct MonthInReview {
    pub month: usize,
    pub year: i32,
    pub album_of_the_month: String,
    pub runners_up: Vec<String>,
    pub tracks_of_the_month: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Rating {
    pub score: Option<i32>,
    pub reviewed_on: i64,
    pub review: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ListenedAlbum {
    pub spotify_id: String,
    pub genre: Option<String>,
    pub first_listened: String,
    pub ratings: Vec<Rating>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ListenedTrack {
    pub spotify_id: String,
    pub score: i32,
}

#[derive(Debug)]
pub struct AlbumData {
    pub title: String,
    pub artist: String,
    pub spotify_link: String,
    pub cover_link: String,
    pub release: String,
}

#[derive(Debug)]
pub struct TrackData {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub spotify_link: String,
    pub cover_link: String,
    pub release: String,
}

pub async fn album_data(album_id: String, access_token: String) -> AlbumData {
    let url = format!("https://api.spotify.com/v1/albums/{album_id}");
    let request = reqwest::Client::new().get(url).bearer_auth(access_token);

    #[derive(Deserialize)]
    struct Image {
        url: String,
    }

    #[derive(Deserialize)]
    struct ExternalUrls {
        spotify: String,
    }

    #[derive(Deserialize)]
    struct Artist {
        name: String,
    }

    #[derive(Deserialize)]
    struct Response {
        name: String,
        images: Vec<Image>,
        release_date: String,
        release_date_precision: String,
        external_urls: ExternalUrls,
        artists: Vec<Artist>,
    }

    let response = request.send().await.unwrap();
    let response: Response = response.json().await.unwrap();

    let release = match response.release_date_precision.deref() {
        "year" => response.release_date.clone(),
        "month" => {
            let date = NaiveDate::parse_from_str(&response.release_date, "%Y-%m").unwrap();
            date.format("%b %Y").to_string()
        }
        "day" => {
            let date = NaiveDate::parse_from_str(&response.release_date, "%Y-%m-%d").unwrap();
            date.format("%b %d, %Y").to_string()
        }
        _ => unreachable!(),
    };

    AlbumData {
        title: response.name,
        artist: response.artists[0].name.clone(),
        spotify_link: response.external_urls.spotify,
        cover_link: response.images[0].url.clone(),
        release,
    }
}

pub async fn track_data(track_id: String, access_token: String) -> TrackData {
    let url = format!("https://api.spotify.com/v1/tracks/{track_id}");
    let request = reqwest::Client::new().get(url).bearer_auth(access_token);

    #[derive(Deserialize)]
    struct Image {
        url: String,
    }

    #[derive(Deserialize)]
    struct ExternalUrls {
        spotify: String,
    }

    #[derive(Deserialize)]
    struct Artist {
        name: String,
    }

    #[derive(Deserialize)]
    struct Album {
        album_type: String,
        name: String,
    }

    #[derive(Deserialize)]
    struct Response {
        name: String,
        release_date: String,
        release_date_precision: String,
        album: Album,
        artists: Vec<Artist>,
        external_urls: ExternalUrls,
        images: Vec<Image>,
    }

    let response = request.send().await.unwrap();
    let response: Response = response.json().await.unwrap();

    let release = match response.release_date_precision.deref() {
        "year" => response.release_date.clone(),
        "month" => {
            let date = NaiveDate::parse_from_str(&response.release_date, "%Y-%m").unwrap();
            date.format("%b %Y").to_string()
        }
        "day" => {
            let date = NaiveDate::parse_from_str(&response.release_date, "%Y-%m-%d").unwrap();
            date.format("%b %d, %Y").to_string()
        }
        _ => unreachable!(),
    };

    let is_album = &response.album.album_type == "album";

    TrackData {
        title: response.name,
        artist: response.artists[0].name.clone(),
        album: is_album.then_some(response.album.name),
        spotify_link: response.external_urls.spotify,
        cover_link: response.images[0].url.clone(),
        release,
    }
}

#[derive(Clone)]
pub struct SpotifyCredentials {
    client_id: String,
    client_secret: String,
    access_token: String,
    expires_at: i64,
}

impl SpotifyCredentials {
    pub async fn fresh() -> Self {
        let client_id = fs::read_to_string(".spotify_client_id")
            .expect("Missing Spotify client ID")
            .trim()
            .to_owned();
        let client_secret = fs::read_to_string(".spotify_client_secret")
            .expect("Missing Spotify client secret")
            .trim()
            .to_owned();

        let request = reqwest::Client::new()
            .post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(format!(
                "grant_type=client_credentials&client_id={client_id}&client_secret={client_secret}"
            ));

        #[derive(Deserialize)]
        struct Response {
            access_token: String,
            expires_in: i64,
        }

        let response = request
            .send()
            .await
            .expect("Error getting Spotify access token");
        let response: Response = response.json().await.unwrap();

        let access_token = response.access_token;
        let expires_at = Utc::now().timestamp() + response.expires_in - 120;

        Self {
            client_id,
            client_secret,
            access_token,
            expires_at,
        }
    }

    pub async fn refresh(&mut self) -> &mut Self {
        let now = Utc::now().timestamp();
        if now > self.expires_at {
            *self = Self::fresh().await
        }
        self
    }

    pub async fn access_token(&mut self) -> &str {
        &self.refresh().await.access_token
    }
}
