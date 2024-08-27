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
        months_in_review_html.push((html, review.year, review.month));
    }

    months_in_review_html.sort_by_key(|(_, y, m)| (y, m));
    months_in_review_html.reverse();

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

        let score_rating = score_html(score);

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
                (review.0)
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

    let review = album_of_the_month
        .ratings
        .last()
        .and_then(|r| r.review.as_ref())
        .map(|r| review_html(r.to_owned(), index, &spotify_data));

    let runners_up_html = month_in_review.runners_up.iter().map(|r| {
        let album = index.albums.iter().find(|a| &a.spotify_id == r).unwrap();
        let album_data = &spotify_data.albums[&album.spotify_id];
        let score_rating = score_html(album.ratings.last().and_then(|r| r.score));
        let has_review = album.ratings.last().is_some_and(|r| r.review.is_some());

        html! {
            .runner-up {
                .album-inline {
                    img.album-cover src=(album_data.cover_link);
                    .album-info {
                        h4 { (album_data.title) }
                        p.album-subtitle {
                            span.album-artist { (album_data.artist) }
                        }
                        (score_rating)
                    }
                }
                p.album-footer {
                    span.release { "Released " (album_data.release) }
                    @if has_review {
                        a.review href={"/album/" (album.spotify_id)} { "Read review" }
                    }
                }
                p.album-footer {
                    a.listen-link href=(album_data.spotify_link) { "Listen on Spotify" }
                    @if let Some(ref genre) = album.genre {
                        span.genre { (genre) }
                    }
                }

            }
        }
    });

    let tracks_html = month_in_review.tracks_of_the_month.iter().map(|r| {
        let track = index.tracks.iter().find(|t| &t.spotify_id == r);
        let track_data = &spotify_data.tracks[r];
        let score_rating = score_html(track.map(|t| t.score));

        html! {
            .track-of-the-month {
                .track-inline {
                    img.track-cover src=(track_data.cover_link);
                    .track-info {
                        h4 { "“" (track_data.title) "”" }
                        p.track-subtitle {
                            span.track-artist { (track_data.artist) }
                        }
                        (score_rating)
                    }
                }
                p.track-footer {
                    a.listen-link href=(track_data.spotify_link) { "Listen on Spotify" }
                    span.release { "Released " (track_data.release) }
                }
            }
        }
    });

    let body = html! {
        h2 { (month_name) " " (month_in_review.year) }
        p.subtitle { "in music" }
        #album-of-the-month {
            p.label { "ALBUM OF THE MONTH" }
            img src=(album_of_the_month_info.cover_link);
            p.album-title { (album_of_the_month_info.title) }
            p.album-artist { (album_of_the_month_info.artist) }
            @if let Some(review) = review {
                #review {
                    (review)
                }
            }
            a.listen href=(album_of_the_month_info.spotify_link) { "Listen on Spotify" }
        }
        h3 { "Runners up" }
        #runners-up {
            @for runner_up in runners_up_html {
                (runner_up)
            }
        }
        h3 { "Songs of the month" }
        #tracks {
            @for track in tracks_html {
                (track)
            }
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

pub async fn album_review(
    index: &Index,
    album: String,
    access_token: &str,
    headers: &HeaderMap,
) -> Markup {
    let spotify_data = SpotifyData::from_file(index, access_token.to_owned()).await;

    let review = index
        .albums
        .iter()
        .find(|a| a.spotify_id == album)
        .and_then(|a| a.ratings.last().map(|r| (a, r)))
        .and_then(|(a, r)| r.review.as_ref().map(|r| (a, r)));

    let Some((album, review)) = review else {
        return html! { "Album not found" };
    };

    let review = review_html(review.to_owned(), index, &spotify_data);
    let album_data = &spotify_data.albums[&album.spotify_id];
    let score_rating = score_html(album.ratings.last().unwrap().score);

    let body = html! {
        h2 { "Album review" }
        #album-review {
            img src=(album_data.cover_link);
            p.album-title { (album_data.title) }
            p.album-artist { (album_data.artist) }
            #review {
                (review)
            }
            (score_rating)
            a.listen href=(album_data.spotify_link) { "Listen on Spotify" }
        }
    };

    let body = wrappers::standard(body, vec![], None);

    wrappers::universal(body, headers, "review", &album_data.title, false)
}

pub fn score_html(score: Option<i32>) -> Markup {
    if let Some(score) = score {
        match score {
            10..=12 => html! {
                span.score-rating.perfect {
                    (score)
                    span.explanation { (RATINGS[score as usize - 1]) }
                }
            },
            7..=9 => html! {
                span.score-rating.great {
                    (score)
                    span.explanation { (RATINGS[score as usize - 1]) }
                }
            },
            _ => html! {
                span.score-rating {
                    (score)
                    span.explanation { (RATINGS[score as usize - 1]) }
                }
            },
        }
    } else {
        html! {
            span.score-rating.none { "Unrated" }
        }
    }
}

pub fn track_ids_in_review(review: &str) -> Vec<String> {
    let track_regex = Regex::new("%(.+?)%").unwrap();
    track_regex
        .captures_iter(review)
        .map(|c| c[1].to_owned())
        .collect()
}

pub fn review_html(mut review: String, index: &Index, data: &SpotifyData) -> Markup {
    let track_regex = Regex::new("%(.+?)%").unwrap();

    while let Some(capture) = track_regex.captures(&review) {
        let track_id = &capture[1];

        let track = &data.tracks[track_id];
        let score = index
            .tracks
            .iter()
            .find(|t| t.spotify_id == track_id)
            .map(|r| r.score);
        let score_rating = score_html(score);

        let track_html = html! {
            a.track-embed href=(track.spotify_link) {
                span.title {
                    "“" (track.title) "”"
                }
                @if score.is_some() {
                    (score_rating)
                }
            }
        };
        let track_text = track_html.0;

        review = review.replace(&capture[0], &track_text);
    }

    html! {
        @for line in review.lines() {
            p.line { (PreEscaped(line)) }
        }
    }
}
