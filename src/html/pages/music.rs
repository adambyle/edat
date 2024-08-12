use std::{collections::HashMap, future::{Future, IntoFuture}, ops::Deref};

use crate::html::music::{album_data, track_data, AlbumData, SpotifyCredentials, TrackData};

use super::*;

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub fn music(index: &mut Index, headers: &HeaderMap) -> Markup {
    let mut months_in_review_html = Vec::with_capacity(index.months_in_review.len());

    let mut covers_needed = HashMap::new();
    for review in &index.months_in_review {
        covers_needed
    }

    for review in &index.months_in_review {
        let html = html! {
            .month-in-review {
                h3 { (MONTHS[review.month]) " " (review.year) }
                p.subtitle { "Month in review" }
                .album-of-the-month {
                    img.album-cover src=()
                }
            }
        };
        months_in_review_html.push(html);
    }

    let body = html! {
        h2.page-title { "Music reviews" }
        #months-in-review {
            @for review in months_in_review_html {
                (review)
            }
        }
    };

    let body = wrappers::standard(body, vec![], None);

    wrappers::universal(body, headers, "music", "Music", false)
}

pub struct SpotifyDataManager {
    albums: HashMap<String, Box<dyn IntoFuture<Output = AlbumData> + 'static>>,
    tracks: HashMap<String, Box<dyn Future<Output = TrackData> + 'static>>,
}

impl SpotifyDataManager {
    pub fn new() -> Self {
        Self {
            albums: HashMap::new(),
            tracks: HashMap::new(),
        }
    }

    pub async fn reserve_album(&mut self, id: String, access_token: String) {
        self.albums
            .entry(id.clone())
            .or_insert_with(|| Box::new(album_data(id, access_token)));
    }

    pub async fn reserve_track(&mut self, id: String, access_token: String) {
        self.tracks
            .entry(id.clone())
            .or_insert_with(|| Box::new(track_data(id, access_token)));
    }

    pub async fn get_album(&self, id: String) -> AlbumData {
        let album = self.albums[&id].deref();
        album.await
    }
}
