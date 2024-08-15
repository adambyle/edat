use chrono::NaiveDate;
use regex::Regex;

use crate::{data, html::music::SpotifyData};

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

const RATINGS: [&str; 12] = [
    "Disastrous",
    "Bad",
    "Disappointing",
    "Okay",
    "Satisfactory",
    "Decent",
    "Good",
    "Great",
    "Excellent",
    "Outstanding",
    "Masterful",
    "Perfect",
];

pub async fn music(index: &Index, access_token: &str, headers: &HeaderMap) -> Markup {
    let mut months_in_review_html = Vec::with_capacity(index.months_in_review.len());
    let mut albums_html = Vec::with_capacity(index.albums.len());

    let spotify_data = SpotifyData::from_file(index, access_token.to_owned()).await;

    for review in &index.months_in_review {
        let album_data = &spotify_data.albums[&review.album_of_the_month];

        let html = html! {
            a.month-in-review href={"/mir/" (review.year) "-" (review.month)} {
                h3 { (MONTHS[review.month]) " " (review.year) }
                .album-of-the-month {
                    img.album-cover src=(album_data.cover_link);
                    .album-info {
                        .album-subtitle { "ALBUM OF THE MONTH" }
                        h4 { (album_data.title) }
                        .album-artist { (album_data.artist) }
                    }
                }
                p.footer { "See runners-up and tracks of the month" }
            }
        };
        months_in_review_html.push(html);
    }

    let mut albums = index.albums.clone();
    albums.sort_by_key(|a| NaiveDate::parse_from_str(&a.first_listened, "%Y-%m-%d").unwrap());
    albums.reverse();

    for album in &albums {
        let album_data = &spotify_data.albums[&album.spotify_id];

        let rating = album.ratings.last();
        let score = rating.and_then(|r| r.score);
        let has_review = rating.is_some_and(|r| r.review.is_some());
        let summary = rating.and_then(|r| r.summary.clone());

        let listened = NaiveDate::parse_from_str(&album.first_listened, "%Y-%m-%d").unwrap();
        let listened = data::date_string(&listened);

        let score_rating = if let Some(score) = score {
            match score {
                10..=12 => html! {
                    p.score-rating.perfect {
                        (score)
                        span.explanation { (RATINGS[score as usize - 1]) }
                    }
                },
                7..=9 => html! {
                    p.score-rating.great {
                        (score)
                        span.explanation { (RATINGS[score as usize - 1]) }
                    }
                },
                _ => html! {
                    p.score-rating {
                        (score)
                        span.explanation { (RATINGS[score as usize - 1]) }
                    }
                },
            }
        } else {
            html! {
                p.score-rating.none { "Unrated" }
            }
        };

        let html = html! {
            .album {
                h4 { (album_data.title) }
                p.album-subtitle {
                    span.album-artist { (album_data.artist) }
                }
                .album-inline {
                    img.album-cover src=(album_data.cover_link);
                    .album-info {
                        (score_rating)
                        .summary {
                            @if let Some(summary) = summary {
                                p { (summary) }
                            }
                        }
                        @if has_review {
                            a.review href={"/album/" (album.spotify_id)} { "Read review" }
                        }
                    }
                }
                p.album-footer {
                    span.release { "Released " (album_data.release) }
                    span.listened { "Listened " (listened) }
                }
                p.album-footer {
                    a.listen-link href=(album_data.spotify_link) { "Listen on Spotify" }
                    @if let Some(ref genre) = album.genre {
                        span.genre { (genre) }
                    }
                }
            }
        };
        albums_html.push(html);
    }

    let body = html! {
        h2 { "Music reviews" }
        #months-in-review {
            @for review in months_in_review_html {
                (review)
            }
        }
        h3.section-header { "Recent albums" }
        #albums {
            @for album in albums_html {
                (album)
            }
        }

    };

    let body = wrappers::standard(body, vec![], None);

    wrappers::universal(body, headers, "music", "Music", false)
}

pub async fn month_in_review(
    index: &Index,
    month: String,
    access_token: &str,
    headers: &HeaderMap,
) -> Markup {
    let month_in_review = index
        .months_in_review
        .iter()
        .find(|m| format!("{}-{}", m.year, m.month) == month);
    let Some(month_in_review) = month_in_review else {
        return html! {
            h1 { "Month not found" }
        };
    };

    let spotify_data = SpotifyData::from_file(index, access_token.to_owned()).await;

    let month_name = MONTHS[month_in_review.month];

    let album_of_the_month = index
        .albums
        .iter()
        .find(|a| a.spotify_id == month_in_review.album_of_the_month)
        .unwrap();
    let album_of_the_month_info = &spotify_data.albums[&album_of_the_month.spotify_id];

    let body = html! {
        h2 { (month_name) " " (month_in_review.year) }
        p.subtitle { "in music" }
        #album-of-the-month {
            p.label { "ALBUM OF THE MONTH" }
            img src=(album_of_the_month_info.cover_link);
            p.album-title { (album_of_the_month_info.title) }
            p.album-artist { (album_of_the_month_info.artist) }
        }
    };

    let body = wrappers::standard(body, vec![], None);

    wrappers::universal(
        body,
        headers,
        "mir",
        &format!("{month_name} in music"),
        false,
    )
}

pub async fn review(mut review: String, index: &Index, data: &SpotifyData) -> Markup {
    let track_regex = Regex::new("%(.+?)%").unwrap();

    while let Some(capture) = track_regex.captures(&review) {
        let capture = capture.get(0).unwrap();
        let track_id = capture.as_str();

        let track = &data.tracks[track_id];
        let score = index
            .tracks
            .iter()
            .find(|t| t.spotify_id == track_id)
            .map(|r| r.score);
        let track_html = html! {
            a.track_embed href=(track.spotify_link) {
                span.title {
                    (track.title)
                }
                @if let Some(score) = score {
                    span.score { (score) }
                }
            }
        };
        let track_text = track_html.0;

        review = review.replace(&capture.as_str(), &track_text);
    }

    todo!()
}
