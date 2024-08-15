use std::{collections::HashMap, fs, ops::Deref};

use chrono::{NaiveDate, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::html::pages::music::track_ids_in_review;

use super::Index;

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
    pub summary: Option<String>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct AlbumData {
    pub title: String,
    pub artist: String,
    pub spotify_link: String,
    pub cover_link: String,
    pub release: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackData {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub spotify_link: String,
    pub cover_link: String,
    pub release: String,
}

pub async fn album_data(album_id: &str, access_token: &str) -> AlbumData {
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
    let text = response.text().await.unwrap();
    println!("Response: {text}");
    let response: Response = serde_json::from_str(&text)
        .expect(format!("Could not get album data for {album_id}",).as_str());

    let release = match response.release_date_precision.deref() {
        "year" => response.release_date.clone(),
        "month" => {
            let date = NaiveDate::parse_from_str(&response.release_date, "%Y-%m").unwrap();
            date.format("%b %Y").to_string()
        }
        "day" => {
            let date = NaiveDate::parse_from_str(&response.release_date, "%Y-%m-%d").unwrap();
            date.format("%b %-d, %Y").to_string()
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

pub async fn track_data(track_id: &str, access_token: &str) -> TrackData {
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
        release_date: String,
        release_date_precision: String,
        images: Vec<Image>,
    }

    #[derive(Deserialize)]
    struct Response {
        name: String,
        album: Album,
        artists: Vec<Artist>,
        external_urls: ExternalUrls,
    }

    let response = request.send().await.unwrap();

    let response: Response = response.json().await.unwrap();

    let release = match response.album.release_date_precision.deref() {
        "year" => response.album.release_date.clone(),
        "month" => {
            let date = NaiveDate::parse_from_str(&response.album.release_date, "%Y-%m").unwrap();
            date.format("%b %Y").to_string()
        }
        "day" => {
            let date = NaiveDate::parse_from_str(&response.album.release_date, "%Y-%m-%d").unwrap();
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
        cover_link: response.album.images[0].url.clone(),
        release,
    }
}

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

#[derive(Deserialize, Serialize)]
pub struct SpotifyData {
    pub albums: HashMap<String, AlbumData>,
    pub tracks: HashMap<String, TrackData>,
    last_updated: i64,
}

impl SpotifyData {
    pub async fn refresh_file(index: &Index, access_token: String) -> Self {
        let data = tokio::fs::read_to_string("content/spotify.json")
            .await
            .unwrap();
        let mut data: Self = serde_json::from_str(&data).unwrap();

        let track_regex = Regex::new("%(.+?)%").unwrap();

        for track in &index.tracks {
            let access_token = access_token.clone();
            let id = track.spotify_id.clone();
            if (!data.tracks.contains_key(&id)) {
                let track_data = track_data(&id, &access_token).await;
                data.tracks.insert(id, track_data);
            }
        }

        for album in &index.albums {
            {
                let access_token = access_token.clone();
                let id = album.spotify_id.clone();
                if (!data.albums.contains_key(&id)) {
                    let album_data = album_data(&id, &access_token).await;
                    data.albums.insert(id, album_data);
                }
            }

            if let Some(review) = album.ratings.last().and_then(|r| r.review.as_ref()) {
                for track_id in track_ids_in_review(review) {
                    let access_token = access_token.clone();

                    if (!data.tracks.contains_key(&track_id)) {
                        println!("From review: {}", &track_id);

                        let track_data = track_data(&track_id, &access_token).await;
                        data.tracks.insert(track_id, track_data);
                    }
                }
            }
        }

        for month in &index.months_in_review {
            for track in &month.tracks_of_the_month {
                let access_token = access_token.clone();
                let track = track.clone();

                if (!data.tracks.contains_key(&track)) {
                    let track_data = track_data(&track, &access_token).await;
                    data.tracks.insert(track, track_data);
                }
            }
        }

        tokio::fs::write(
            "content/spotify.json",
            serde_json::to_string(&data).unwrap(),
        )
        .await
        .unwrap();

        data
    }

    pub async fn from_file(index: &Index, access_token: String) -> Self {
        let data = tokio::fs::read_to_string("content/spotify.json")
            .await
            .unwrap();
        let data: Self = serde_json::from_str(&data).unwrap();
        let week_in_seconds = 7 * 24 * 60 * 60;

        if Utc::now().timestamp() - data.last_updated > week_in_seconds {
            Self::refresh_file(index, access_token).await
        } else {
            data
        }
    }
}
