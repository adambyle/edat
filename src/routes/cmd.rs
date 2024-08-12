use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use axum::http::Extensions;
use chrono::{NaiveDate, Utc};
use indexmap::IndexMap;
use serde::Deserialize;

use super::*;

pub async fn image_upload(
    headers: HeaderMap,
    ReqPath(mut file_name): ReqPath<String>,
    body: Bytes,
) -> impl IntoResponse {
    let image_path = format!("content/images/{file_name}");
    let image_path = Path::new(&image_path);
    let is_jpeg = headers
        .get("Content-Type")
        .is_some_and(|c| c == "image/jpeg");

    file_name = file_name.to_lowercase();
    if file_name.ends_with(".jpeg") {
        file_name = file_name.replace(".jpeg", ".jpg");
    }

    if !file_name.ends_with(".jpg") || image_path.exists() || !is_jpeg {
        return html::cmd::image_error(&file_name);
    }
    let mut file = File::create(image_path).unwrap();
    file.write(&body[..]).unwrap();
    html::cmd::image_success(&file_name)
}

#[derive(Deserialize)]
pub enum Body {
    GetSection {
        id: u32,
    },
    NewSection {
        date: String,
    },
    SetSection {
        id: u32,
        heading: String,
        description: String,
        summary: String,
        date: String,
    },
    SetNewSection {
        position: Position<String, u32>,
        heading: String,
        description: String,
        summary: String,
        date: String,
    },
    DeleteSection {
        id: u32,
    },
    MoveSection {
        id: u32,
        position: Position<String, u32>,
    },
    SectionStatus {
        id: u32,
        status: section::Status,
    },
    GetEntry {
        id: String,
    },
    NewEntry,
    SetEntry {
        id: String,
        title: String,
        description: String,
        summary: String,
    },
    SetNewEntry {
        title: String,
        position: Position<(String, usize), String>,
        description: String,
        summary: String,
    },
    DeleteEntry {
        id: String,
    },
    MoveEntry {
        id: String,
        position: Position<(String, usize), String>,
    },
    GetVolume {
        id: String,
    },
    NewVolume,
    SetVolume {
        id: String,
        title: String,
        subtitle: String,
    },
    SetNewVolume {
        position: Position<(), String>,
        title: String,
        subtitle: String,
    },
    DeleteVolume {
        id: String,
    },
    MoveVolume {
        id: String,
        position: Position<(), String>,
    },
    VolumeContentType {
        id: String,
        kind: volume::Kind,
    },
    GetUser {
        id: String,
    },
    SetUser {
        id: String,
        first_name: String,
        last_name: String,
    },
    SetNewUser {
        first_name: String,
        last_name: String,
    },
    NewUser,
    UserPrivilege {
        id: String,
        privilege: UserPrivilege,
    },
    AddUserCode {
        id: String,
        code: String,
    },
    RemoveUserCode {
        id: String,
        code: String,
    },
    Volumes,
    NextSectionId,
    Images,
    GetContent {
        id: u32,
    },
    SetContent {
        id: u32,
        content: String,
    },
    GetIntro {
        id: Option<String>,
    },
    SetIntro {
        id: Option<String>,
        content: String,
    },
    InitUser {
        id: String,
    },
    NewReview,
    SetNewReview {
        album_id: String,
        genre: Option<String>,
        score: Option<i32>,
        review: Option<String>,
        first_listened: Option<String>,
    },
    SetTrackReview {
        track_id: String,
        score: i32,
    },
    NewMonthInReview,
    SetMonthInReview {
        albums: Vec<String>,
        tracks: Vec<String>,
        month: usize,
        year: i32,
    },
}

pub async fn cmd(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<Body>,
) -> Result<Response, maud::Markup> {
    use html::cmd as cmd_html;
    let mut index = state.index.lock().unwrap();

    let user = get_cookie(&headers, "edat_user").ok_or(cmd_html::unauthorized())?;

    fn map_err_html<T>(result: DataResult<T>) -> Result<T, maud::Markup> {
        result.map_err(|err| match err {
            DataError::DuplicateId(id) => cmd_html::duplicate(id),
            DataError::MissingResource(kind, id) => cmd_html::missing(kind, id),
        })
    }

    fn user_info(user: User) -> cmd_html::UserInfo {
        let codes = user.codes().join(" ");

        // Transform the list of read sections into a list of read entries (and their date).
        let mut entry_history = IndexMap::new();
        for (s, h) in user.history() {
            if let Some(entry) = entry_history.get_mut(s.parent_entry_id()) {
                *entry = std::cmp::max(*entry, h.timestamp());
            } else {
                entry_history.insert(s.parent_entry_id().to_owned(), h.timestamp());
            }
        }
        entry_history.sort_by_cached_key(|_, &t| t);
        let history = entry_history
            .into_iter()
            .map(|(id, timestamp)| cmd_html::UserHistoryEntry {
                entry: id,
                timestamp,
            })
            .collect();

        let preferences = user
            .preferences()
            .iter()
            .map(|p| cmd_html::UserPreference {
                setting: p.0.to_owned(),
                switch: p.1.to_owned(),
            })
            .collect();
        let widgets = user.widgets().join(" ");
        cmd_html::UserInfo {
            codes,
            first_name: user.first_name().to_owned(),
            last_name: user.last_name().to_owned(),
            history,
            preferences,
            privilege: format!("{:?}", user.privilege()),
            widgets,
        }
    }

    fn volume_info(volume: Volume) -> cmd_html::VolumeInfo {
        let entries = volume
            .entries()
            .map(|e| cmd_html::VolumeEntry {
                id: e.id().to_owned(),
                description: e.description().to_owned(),
            })
            .collect();
        cmd_html::VolumeInfo {
            id: volume.id().to_owned(),
            title: volume.title().to_owned(),
            subtitle: volume.subtitle().cloned().unwrap_or_else(String::new),
            owner: volume.owner_id().to_owned(),
            content_type: format!("{:?}", volume.kind()),
            entries,
            volume_count: volume.parts_count(),
        }
    }

    fn entry_info(entry: Entry) -> cmd_html::EntryInfo {
        let sections = entry
            .sections()
            .map(|s| cmd_html::EntrySection {
                id: s.id().to_owned(),
                description: s.description().to_owned(),
            })
            .collect();
        cmd_html::EntryInfo {
            id: entry.id().to_owned(),
            title: entry.title().to_owned(),
            description: entry.description().to_owned(),
            summary: entry.summary().to_owned(),
            author: entry.author_id().to_owned(),
            parent_volume: (
                entry.parent_volume_id().to_owned(),
                entry.parent_volume_part(),
            ),
            sections,
        }
    }

    fn section_info(section: Section) -> cmd_html::SectionInfo {
        let in_entry = section.index_in_parent();
        let perspectives = section
            .perspective_ids()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let comments = section
            .threads()
            .iter()
            .flat_map(|t| t.comments.iter())
            .map(|c| cmd_html::SectionComment {
                author: c.author.id().to_owned(),
                contents: c.content.last().unwrap().to_owned(),
                timestamp: c.timestamp,
            })
            .collect();
        cmd_html::SectionInfo {
            id: section.id(),
            heading: section.heading().cloned().unwrap_or_else(String::new),
            description: section.description().to_owned(),
            summary: section.summary().to_owned(),
            date: section.date().format("%Y-%m-%d").to_string(),
            parent_entry: section.parent_entry_id().to_owned(),
            in_entry: (in_entry, section.parent_entry().section_count()),
            length: section.length(),
            status: format!("{:?}", section.status()),
            perspectives,
            comments,
        }
    }

    fn volumes(index: &Index) -> cmd_html::Volumes {
        let volumes = index
            .volumes()
            .map(|v| {
                (
                    v.id().to_owned(),
                    v.subtitle().cloned().unwrap_or_else(String::new),
                )
            })
            .collect();
        cmd_html::Volumes(volumes)
    }

    use Body as B;
    Ok(match body {
        B::AddUserCode { id, code } => {
            let mut user = map_err_html(index.user_mut(id))?;
            user.add_code(code.to_lowercase());
            cmd_html::user(user_info(user.as_immut()))
        }
        B::DeleteEntry { id } => {
            let entry = map_err_html(index.entry_mut(id))?;
            let parent_volume = entry.parent_volume_id().to_owned();
            entry.remove();
            let volume = index.volume(parent_volume).unwrap();
            cmd_html::volume(volume_info(volume))
        }
        B::DeleteSection { id } => {
            let section = map_err_html(index.section_mut(id))?;
            let parent_entry = section.parent_entry_id().to_owned();
            section.remove();
            let entry = index.entry(parent_entry).unwrap();
            cmd_html::entry(entry_info(entry))
        }
        B::DeleteVolume { id } => {
            let volume = map_err_html(index.volume_mut(id))?;
            volume.remove();
            cmd_html::volumes(volumes(&index))
        }
        B::GetContent { id } => {
            let section = map_err_html(index.section(id))?;
            let content = section.content();
            cmd_html::content(id.to_string(), content)
        }
        B::GetEntry { id } => {
            let entry = map_err_html(index.entry(id))?;
            cmd_html::entry(entry_info(entry))
        }
        B::GetIntro { id } => match id {
            None => {
                let content = fs::read_to_string("content/edat.intro").unwrap();
                cmd_html::content("edat".to_owned(), content)
            }
            Some(id) => {
                let volume = map_err_html(index.volume(id.clone()))?;
                let content = volume.intro();
                cmd_html::content(id, content)
            }
        },
        B::GetSection { id } => {
            let section = map_err_html(index.section(id))?;
            cmd_html::section(section_info(section))
        }
        B::GetUser { id } => {
            let user = map_err_html(index.user(id))?;
            cmd_html::user(user_info(user))
        }
        B::GetVolume { id } => {
            let volume = map_err_html(index.volume(id))?;
            cmd_html::volume(volume_info(volume))
        }
        B::Images => cmd_html::images(),
        B::InitUser { id } => {
            let mut user = map_err_html(index.user_mut(id))?;
            user.init();

            if user.widgets().len() == 0 {
                user.set_widgets(vec![
                    "recent-widget".to_owned(),
                    "library-widget".to_owned(),
                    "last-widget".to_owned(),
                    "conversations-widget".to_owned(),
                    "random-widget".to_owned(),
                    "extras-widget".to_owned(),
                    "search-widget".to_owned(),
                ]);
            }

            cmd_html::user(user_info(user.as_immut()))
        }
        B::MoveEntry { id, position } => {
            let mut entry = map_err_html(index.entry_mut(id))?;
            map_err_html(entry.move_to(position))?;
            cmd_html::entry(entry_info(entry.as_immut()))
        }
        B::MoveSection { id, position } => {
            let mut section = map_err_html(index.section_mut(id))?;
            map_err_html(section.move_to(position))?;
            cmd_html::section(section_info(section.as_immut()))
        }
        B::MoveVolume { id, position } => {
            let mut volume = map_err_html(index.volume_mut(id))?;
            map_err_html(volume.move_to(position))?;
            cmd_html::volume(volume_info(volume.as_immut()))
        }
        B::NextSectionId => return Ok(Json(index.next_section_id()).into_response()),
        B::NewEntry => cmd_html::edit_entry(None),
        B::NewMonthInReview => cmd_html::add_month_in_review(),
        B::NewReview => cmd_html::add_review(),
        B::NewSection { date } => cmd_html::edit_section(None, &date),
        B::NewUser => cmd_html::edit_user(None),
        B::NewVolume => cmd_html::edit_volume(None),
        B::RemoveUserCode { id, code } => {
            let mut user = map_err_html(index.user_mut(id))?;
            user.remove_code(&code.to_lowercase());
            cmd_html::user(user_info(user.as_immut()))
        }
        B::SectionStatus { id, status } => {
            let mut section = map_err_html(index.section_mut(id))?;
            section.set_status(status);
            cmd_html::section(section_info(section.as_immut()))
        }
        B::SetContent { id, content } => {
            let mut section = map_err_html(index.section_mut(id))?;
            section.set_content(&content);
            cmd_html::section(section_info(section.as_immut()))
        }
        B::SetEntry {
            id,
            title,
            description,
            summary,
        } => {
            let mut entry = map_err_html(index.entry_mut(id))?;
            map_err_html(entry.set_title(&title))?;
            entry.set_description(&description);
            entry.set_summary(&summary);
            cmd_html::entry(entry_info(entry.as_immut()))
        }
        B::SetIntro { id, content } => {
            if let Some(id) = id {
                let mut volume = map_err_html(index.volume_mut(id))?;
                volume.set_intro(&content);
                cmd_html::volume(volume_info(volume.as_immut()))
            } else {
                fs::write("content/edat.intro", &content).unwrap();
                cmd_html::content("edat".to_owned(), content)
            }
        }
        B::SetMonthInReview {
            albums,
            tracks,
            month,
            year,
        } => {
            let month = month - 1;
            let existing_month_in_review = index
                .months_in_review
                .iter_mut()
                .find(|m| m.year == year && m.month == month);
            let mut albums = albums.into_iter();
            let best_album = albums.next().unwrap();
            let runners_up = albums.collect();
            if let Some(existing_month_in_review) = existing_month_in_review {
                existing_month_in_review.album_of_the_month = best_album;
                existing_month_in_review.runners_up = runners_up;
                existing_month_in_review.tracks_of_the_month = tracks;
            } else {
                index.months_in_review.push(MonthInReview {
                    year,
                    month,
                    album_of_the_month: best_album,
                    runners_up,
                    tracks_of_the_month: tracks,
                });
            }
            cmd_html::ok()
        }
        B::SetNewEntry {
            title,
            position,
            description,
            summary,
        } => {
            let entry = map_err_html(index.create_entry(
                &title,
                &description,
                &summary,
                user.to_owned(),
                position,
            ))?;
            cmd_html::entry(entry_info(entry.as_immut()))
        }
        B::SetNewReview {
            album_id,
            genre,
            score,
            review,
            first_listened,
        } => {
            let now = Utc::now().timestamp();
            let existing_album = index.albums.iter_mut().find(|a| a.spotify_id == album_id);
            if let Some(existing_album) = existing_album {
                let existing_rating = existing_album.ratings.last();
                let existing_score = existing_rating.and_then(|r| r.score);
                let existing_review = existing_rating.and_then(|r| r.review.clone());

                let rating = Rating {
                    review: review.or(existing_review),
                    score: score.or(existing_score),
                    reviewed_on: now,
                };

                existing_album.ratings.push(rating);
            } else {
                let rating = Rating {
                    review,
                    score,
                    reviewed_on: now,
                };

                let first_listened =
                    first_listened.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

                index.albums.push(ListenedAlbum {
                    spotify_id: album_id,
                    genre,
                    first_listened,
                    ratings: vec![rating],
                });
            }
            index.save_all();
            cmd_html::ok()
        }
        B::SetNewSection {
            position,
            heading,
            description,
            summary,
            date,
        } => {
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|_| cmd_html::bad_date(&date))?;
            let section = map_err_html(index.create_section(
                (!heading.is_empty()).then_some(&heading),
                &description,
                &summary,
                date,
                position,
            ))?;
            cmd_html::section(section_info(section.as_immut()))
        }
        B::SetNewUser {
            first_name,
            last_name,
        } => {
            let user = map_err_html(index.create_user(first_name, last_name))?;
            cmd_html::user(user_info(user.as_immut()))
        }
        B::SetNewVolume {
            position,
            title,
            subtitle,
        } => {
            let volume = map_err_html(index.create_volume(
                &title,
                (!subtitle.is_empty()).then_some(&subtitle),
                user.to_owned(),
                position,
            ))?;
            cmd_html::volume(volume_info(volume.as_immut()))
        }
        B::SetSection {
            id,
            heading,
            description,
            summary,
            date,
        } => {
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|_| cmd_html::bad_date(&date))?;
            let mut section = map_err_html(index.section_mut(id))?;
            section.set_heading((!heading.is_empty()).then_some(&heading));
            section.set_description(&description);
            section.set_summary(&summary);
            section.set_date(date);
            cmd_html::section(section_info(section.as_immut()))
        }
        B::SetTrackReview { track_id, score } => {
            let existing_track_review = index.tracks.iter_mut().find(|t| t.spotify_id == track_id);
            if let Some(existing_track_review) = existing_track_review {
                existing_track_review.score = score;
            } else {
                index.tracks.push(ListenedTrack {
                    spotify_id: track_id,
                    score,
                });
            }
            index.save_all();
            cmd_html::ok()
        }
        B::SetUser {
            id,
            first_name,
            last_name,
        } => {
            let mut user = map_err_html(index.user_mut(id))?;
            map_err_html(user.set_name(first_name, last_name))?;
            cmd_html::user(user_info(user.as_immut()))
        }
        B::SetVolume {
            id,
            title,
            subtitle,
        } => {
            let mut volume = map_err_html(index.volume_mut(id))?;
            map_err_html(volume.set_title(&title))?;
            volume.set_subtitle((!subtitle.is_empty()).then_some(&subtitle));
            cmd_html::volume(volume_info(volume.as_immut()))
        }
        B::UserPrivilege { id, privilege } => {
            let mut user = map_err_html(index.user_mut(id))?;
            user.set_privilege(privilege);
            cmd_html::user(user_info(user.as_immut()))
        }
        B::VolumeContentType { id, kind } => {
            let mut volume = map_err_html(index.volume_mut(id))?;
            volume.set_kind(kind);
            cmd_html::volume(volume_info(volume.as_immut()))
        }
        B::Volumes => cmd_html::volumes(volumes(&index)),
    }
    .into_response())
}
