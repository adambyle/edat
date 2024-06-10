use std::collections::HashMap;
use std::fs::{self, read_to_string};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::{fs::File, path::Path};

use axum::body::Bytes;
use axum::extract::{Path as ReqPath, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::{Datelike, NaiveDate, Utc};
use indexmap::IndexMap;
use rand::Rng;
use serde::Deserialize;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::html::home::Widget;
use crate::{data::*, html, AppState};

// TODO refactor and deal with unwraps.

pub async fn script(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file("static/scripts", file_name, "text/javascript")
}

pub async fn style(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file("static/styles", file_name, "text/css")
}

pub async fn image(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file("content/images", file_name, "image/jpeg")
}

pub async fn image_upload(
    headers: HeaderMap,
    ReqPath(file_name): ReqPath<String>,
    body: Bytes,
) -> impl IntoResponse {
    let image_path = format!("content/images/{file_name}");
    let image_path = Path::new(&image_path);
    let is_jpeg = headers
        .get("Content-Type")
        .is_some_and(|c| c == "image/jpeg");
    if !file_name.ends_with(".jpg") || image_path.exists() || !is_jpeg {
        return html::terminal::image_error(&file_name);
    }
    let mut file = File::create(image_path).unwrap();
    file.write(&body[..]).unwrap();
    html::terminal::image_success(&file_name)
}

pub async fn login(
    State(state): State<AppState>,
    ReqPath((name, code)): ReqPath<(String, String)>,
) -> Response {
    let index = state.index.lock().unwrap();
    let name = name.to_lowercase().replace(char::is_whitespace, "");
    let code = code.to_lowercase();

    // Find a user whose first name matches the input or whose id matches the input.
    for user in index.users() {
        if (name == user.first_name().to_lowercase() || &name == user.id()) && user.has_code(&code)
        {
            return (StatusCode::OK, user.id().to_owned()).into_response();
        }
    }

    StatusCode::UNAUTHORIZED.into_response()
}

pub async fn profile(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let index = state.index.lock().unwrap();
    let user = match login_check(&headers, &index) {
        Ok(user) => user,
        Err(html) => return html,
    };

    // Get the sections read in the last two months.
    let two_months_ago = Utc::now().timestamp() - 60 * 60 * 24 * 60;
    let sections = user
        .history()
        .iter()
        .filter_map(|(s, p)| {
            let Some((progress, timestamp)) = p.progress() else {
                return None;
            };
            if timestamp < two_months_ago {
                return None;
            }

            Some(html::profile::ViewedSection {
                description: s.description().to_owned(),
                timestamp: timestamp,
                entry: s.parent_entry().title().to_owned(),
                id: s.id(),
                index: (s.index_in_parent(), s.parent_entry().section_count()),
                progress: (progress, s.lines()),
            })
        })
        .collect();

    let profile_data = html::profile::ProfileData {
        widgets: user.widgets().to_owned(),
        sections,
    };
    html::profile(&headers, profile_data)
}

pub async fn archive() -> impl IntoResponse {
    let now = Utc::now();
    let archive_path = format!("edat-{}-{}-{}.zip", now.year(), now.month(), now.day());
    let archive_file = File::create(&archive_path).unwrap();
    let mut zip = ZipWriter::new(archive_file);

    fn directory(path: PathBuf, zip: &mut ZipWriter<File>) {
        let options = SimpleFileOptions::default();
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                directory(path, zip);
            } else {
                zip.start_file(path.to_str().unwrap(), options).unwrap();
                let mut file = File::open(path).unwrap();
                io::copy(&mut file, zip).unwrap();
            }
        }
    }

    directory("content".into(), &mut zip);
    directory("users".into(), &mut zip);
    directory("archived".into(), &mut zip);
    zip.finish().unwrap();

    let response = static_file("./", archive_path.clone(), "application/zip");
    fs::remove_file(archive_path).unwrap();
    response
}

#[derive(Deserialize)]
pub struct RegisterBody {
    entries: Vec<String>,
    widgets: Vec<String>,
}

pub async fn widgets(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(widgets): Json<Vec<String>>,
) -> StatusCode {
    let mut index = state.index.lock().unwrap();
    let user = get_cookie(&headers, "edat_user").unwrap();
    let mut user = index.user_mut(user.to_owned()).unwrap();

    user.set_widgets(widgets);

    StatusCode::OK
}

#[derive(Deserialize)]
pub struct ReadQuery {
    progress: Option<usize>,
    entry: bool,
}

pub async fn read(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath(id): ReqPath<String>,
    Query(options): Query<ReadQuery>,
) -> StatusCode {
    let mut index = state.index.lock().unwrap();
    let user = get_cookie(&headers, "edat_user").unwrap();
    let mut user = index.user_mut(user.to_owned()).unwrap();

    if options.entry {
        user.finished_entry(id);
    } else if let Some(progress) = options.progress {
        if let Ok(id) = id.parse() {
            user.reading_section(id, progress);
        }
    } else {
        if let Ok(id) = id.parse() {
            user.finished_section(id);
        }
    }

    StatusCode::OK
}

pub async fn register(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) {
    let mut index = state.index.lock().unwrap();

    // Collect the sections the user has read.
    let mut sections: Vec<u32> = Vec::new();
    if body.entries.get(0).is_some_and(|e| e == "$all") {
        // The user indicated they have read everything.
        sections.extend(index.sections().map(|s| s.id()));
    } else {
        // Load the specific entries.
        for entry in body.entries {
            let Ok(entry) = index.entry(entry) else {
                continue;
            };
            sections.extend(entry.section_ids());
        }
    }

    // Update the user's history and widget preferences.
    let user = get_cookie(&headers, "edat_user").unwrap();
    let mut user = index.user_mut(user.to_owned()).unwrap();
    for section in sections {
        user.finished_section(section);
    }
    user.set_widgets(body.widgets);
    user.init();
}

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let index = state.index.lock().unwrap();
    let user = match login_check(&headers, &index) {
        Ok(user) => user,
        Err(html) => return html,
    };

    // Direct the user to a setup screen if the history is uninitialized.
    if !user.is_init() {
        // Collect the entries a user may specify they have read
        // (from standard journal volumes).
        let volumes = index
            .volumes()
            .filter(|v| v.kind() == volume::Kind::Journal)
            .map(|v| {
                let entries = v
                    .entries()
                    .map(|e| html::setup::Entry {
                        id: e.id().to_owned(),
                        title: e.title().to_owned(),
                        description: e.description().to_owned(),
                    })
                    .collect();
                html::setup::Volume {
                    title: v.title().to_owned(),
                    entries,
                }
            })
            .collect();
        return html::setup(&headers, volumes);
    }

    // Initialize homepage widgets.
    let mut widgets = Vec::new();

    let recent_widget = || {
        // Get all the complete sections from journal volumes.
        let mut sections: Vec<_> = index
            .sections()
            .filter(|s| {
                let parent_entry = s.parent_entry();
                let parent_volume = parent_entry.parent_volume();
                parent_volume.kind() == volume::Kind::Journal
                    && s.status() == section::Status::Complete
            })
            .collect();

        // Sort them by recency.
        sections.sort_by(|a, b| {
            let date_a = a.date();
            let date_b = b.date();
            if date_a == date_b {
                b.id().cmp(&a.id())
            } else {
                date_b.cmp(&date_a)
            }
        });

        // Process the 10 latest sections...
        let recents = sections[..10.min(sections.len())]
            .iter()
            .map(|section| {
                let parent_entry = section.parent_entry();
                let in_entry = section.index_in_parent();
                let previous =
                    (in_entry > 0).then(|| parent_entry.sections().nth(in_entry - 1).unwrap());
                let read = user
                    .history()
                    .iter()
                    .find(|(s, _)| s.id() == section.id())
                    .and_then(|(_, h)| h.timestamp());
                let parent_volume = parent_entry.parent_volume();
                let parent_volume = (
                    parent_volume.title().to_owned(),
                    parent_entry.parent_volume_part(),
                    parent_volume.parts_count(),
                );

                html::home::RecentSection {
                    id: section.id(),
                    parent_volume,
                    parent_entry: parent_entry.title().to_owned(),
                    in_entry: (in_entry + 1, parent_entry.section_ids().len()),
                    date: section.date().format("%Y-%m-%d").to_string(),
                    previous: previous.map(|s| (s.id(), s.description().to_owned())),
                    description: section.description().to_owned(),
                    summary: section.summary().to_owned(),
                    length: section.length_string(),
                    read,
                }
            })
            .collect();
        let expand = user.preferences().get("expand_recents");
        let expand = match expand {
            Some(expand) => expand == "true",
            None => true,
        };
        html::home::RecentWidget {
            sections: recents,
            expand,
        }
    };

    let library_widget = || {
        let volumes = index
            .volumes()
            .filter(|v| v.kind() == volume::Kind::Journal)
            .map(|v| html::home::LibraryVolume {
                title: v.title().to_owned(),
                id: v.id().to_owned(),
                subtitle: v.subtitle().map(|s| s.to_owned()),
                entry_count: v.entry_ids().len(),
            })
            .collect();

        html::home::LibraryWidget {
            volumes,
            title: "The library".to_owned(),
        }
    };

    let extras_widget = || {
        let volumes = index
            .volumes()
            .filter(|v| v.kind() != volume::Kind::Journal && v.kind() != volume::Kind::Featured)
            .map(|v| html::home::LibraryVolume {
                title: v.title().to_owned(),
                id: v.id().to_owned(),
                subtitle: v.subtitle().map(|s| s.to_owned()),
                entry_count: v.entry_ids().len(),
            })
            .collect();

        html::home::LibraryWidget {
            volumes,
            title: "Extras".to_owned(),
        }
    };

    let last_widget = || {
        let section = user
            .history()
            .iter()
            .filter_map(|(s, h)| h.progress().map(|p| (s, p)))
            .next()
            .map(|(s, p)| {
                let entry = s.parent_entry();
                let index = s.index_in_parent();
                html::home::LastSection {
                    entry: entry.title().to_owned(),
                    summary: s.summary().to_owned(),
                    timestamp: p.1,
                    id: s.id(),
                    index: (index, entry.section_count()),
                    progress: (p.0, s.lines()),
                }
            });

        html::home::LastWidget { section }
    };

    let random_widget = || {
        // TODO make random.

        let wrap_entry = |entry: &Entry| {
            let volume = entry.parent_volume();
            let volume_part = (volume.parts_count() > 1).then_some(entry.parent_volume_part());

            html::home::RandomEntry {
                id: entry.id().to_owned(),
                summary: entry.summary().to_owned(),
                title: entry.title().to_owned(),
                volume: volume.title().to_owned(),
                volume_part,
            }
        };

        let unstarted_entries: Vec<_> = index
            .entries()
            .filter(|e| matches!(user.entry_progress(&e), EntryProgress::Unstarted))
            .collect();
        if !unstarted_entries.is_empty() {
            let entry =
                &unstarted_entries[rand::thread_rng().gen_range(0..unstarted_entries.len())];
            return html::home::RandomWidget::Unstarted(wrap_entry(entry));
        }

        let unfinished_entries: Vec<_> = index
            .entries()
            .filter_map(|e| {
                if let EntryProgress::InSection {
                    section_id,
                    section_index,
                    progress,
                    ..
                } = user.entry_progress(&e)
                {
                    Some((e, section_id, section_index, progress))
                } else {
                    None
                }
            })
            .collect();
        if let Some(entry) = unfinished_entries.last() {
            return html::home::RandomWidget::Unfinished {
                entry: wrap_entry(&entry.0),
                section_id: entry.1,
                section_index: entry.2,
                progress: entry.3,
            };
        }

        let mut read_again: Vec<_> = index
            .entries()
            .filter_map(|e| {
                if let EntryProgress::Finished { last_read } = user.entry_progress(&e) {
                    Some((e, last_read))
                } else {
                    None
                }
            })
            .collect();
        read_again.sort_by_key(|e| e.1);
        let entry = &read_again[rand::thread_rng().gen_range(0..read_again.len().min(10))];

        html::home::RandomWidget::ReadAgain {
            entry: wrap_entry(&entry.0),
            last_read: entry.1,
        }
    };

    for widget in user.widgets() {
        let widget: Box<dyn Widget> = match widget.as_ref() {
            "recent-widget" => Box::new(recent_widget()),
            "library-widget" => Box::new(library_widget()),
            "extras-widget" => Box::new(extras_widget()),
            "last-widget" => Box::new(last_widget()),
            "random-widget" => Box::new(random_widget()),
            _ => continue,
        };
        widgets.push(widget);
    }

    let intro = read_to_string("content/edat.intro").unwrap();
    let intro_lines: Vec<&str> = intro.lines().filter(|l| l.len() > 0).collect();
    html::home(&headers, widgets, intro_lines)
}

pub async fn preferences(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<HashMap<String, Option<String>>>,
) {
    let mut index = state.index.lock().unwrap();
    let user = get_cookie(&headers, "edat_user").unwrap();
    let mut user = index.user_mut(user.to_owned()).unwrap();
    for (k, v) in body {
        match v {
            Some(v) => {
                user.set_preference(k, v);
            }
            None => {
                user.remove_preference(&k);
            }
        }
    }
}

fn static_file(folder: &str, file_name: String, content_type: &'static str) -> Response {
    let path = Path::new(folder).join(&file_name);

    match File::open(path.clone()) {
        Ok(mut file) => {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).unwrap();
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!("inline; filename=\"{}\"", file_name),
                    ),
                ],
                contents,
            )
                .into_response()
        }
        Err(_) => {
            println!("Invalid path: {}", path.to_str().unwrap());
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

fn login_check<'index>(
    headers: &HeaderMap,
    index: &'index Index,
) -> Result<User<'index>, maud::Markup> {
    let err = || html::login(headers);

    let Some(username) = get_cookie(headers, "edat_user") else {
        return Err(err());
    };

    index.user(username.to_owned()).map_err(|_| err())
}

pub fn get_cookie<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    let cookie = headers.get("Cookie")?.to_str().unwrap();
    cookie.split(&format!("{key}=")).nth(1)?.split(';').next()
}

pub async fn terminal(headers: HeaderMap, State(state): State<AppState>) -> maud::Markup {
    let index = state.index.lock().unwrap();
    let user = match login_check(&headers, &index) {
        Ok(user) => user,
        Err(html) => return html,
    };
    html::terminal(&headers, user.privilege() == user::Privilege::Owner)
}

mod cmd {
    use serde::Deserialize;

    use crate::data::*;

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
            privilege: user::Privilege,
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
    }
}

pub async fn cmd(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<cmd::Body>,
) -> Result<Response, maud::Markup> {
    use html::terminal;
    let mut index = state.index.lock().unwrap();

    let user = get_cookie(&headers, "edat_user").ok_or(html::terminal::unauthorized())?;

    fn map_err_html<T>(result: DataResult<T>) -> Result<T, maud::Markup> {
        result.map_err(|err| match err {
            DataError::DuplicateId(id) => terminal::duplicate(id),
            DataError::MissingResource(kind, id) => terminal::missing(kind, id),
        })
    }

    fn user_info(user: User) -> terminal::UserInfo {
        let codes = user.codes().join(" ");

        // Transform the list of read sections into a list of read entries (and their date).
        let mut entry_history = IndexMap::new();
        for (s, h) in user.history() {
            if let Some(timestamp) = h.timestamp() {
                if let Some(entry) = entry_history.get_mut(s.parent_entry_id()) {
                    *entry = std::cmp::max(*entry, timestamp);
                } else {
                    entry_history.insert(s.parent_entry_id().to_owned(), timestamp);
                }
            }
        }
        entry_history.sort_by_cached_key(|_, &t| t);
        let history = entry_history
            .into_iter()
            .map(|(id, timestamp)| terminal::UserHistoryEntry {
                entry: id,
                timestamp,
            })
            .collect();

        let preferences = user
            .preferences()
            .iter()
            .map(|p| terminal::UserPreference {
                setting: p.0.to_owned(),
                switch: p.1.to_owned(),
            })
            .collect();
        let widgets = user.widgets().join(" ");
        terminal::UserInfo {
            codes,
            first_name: user.first_name().to_owned(),
            last_name: user.last_name().to_owned(),
            history,
            preferences,
            privilege: format!("{:?}", user.privilege()),
            widgets,
        }
    }

    fn volume_info(volume: Volume) -> terminal::VolumeInfo {
        let entries = volume
            .entries()
            .map(|e| terminal::VolumeEntry {
                id: e.id().to_owned(),
                description: e.description().to_owned(),
            })
            .collect();
        terminal::VolumeInfo {
            id: volume.id().to_owned(),
            title: volume.title().to_owned(),
            subtitle: volume
                .subtitle()
                .map(|s| s.clone())
                .unwrap_or("".to_owned()),
            owner: volume.owner_id().to_owned(),
            content_type: format!("{:?}", volume.kind()),
            entries,
            volume_count: volume.parts_count(),
        }
    }

    fn entry_info(entry: Entry) -> terminal::EntryInfo {
        let sections = entry
            .sections()
            .map(|s| terminal::EntrySection {
                id: s.id().to_owned(),
                description: s.description().to_owned(),
            })
            .collect();
        terminal::EntryInfo {
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

    fn section_info(section: Section) -> terminal::SectionInfo {
        let parent_entry = section.parent_entry();
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
            .map(|c| terminal::SectionComment {
                author: c.author.id().to_owned(),
                contents: c.content.last().unwrap().to_owned(),
                timestamp: c.timestamp,
            })
            .collect();
        terminal::SectionInfo {
            id: section.id(),
            heading: section
                .heading()
                .map(|s| s.clone())
                .unwrap_or("".to_owned()),
            description: section.description().to_owned(),
            summary: section.summary().to_owned(),
            date: section.date().format("%Y-%m-%d").to_string(),
            parent_entry: section.parent_entry_id().to_owned(),
            in_entry: (in_entry, parent_entry.section_count()),
            length: section.length(),
            status: format!("{:?}", section.status()),
            perspectives,
            comments,
        }
    }

    fn volumes(index: &Index) -> terminal::Volumes {
        let volumes = index
            .volumes()
            .map(|v| {
                (
                    v.id().to_owned(),
                    v.subtitle().map(|s| s.clone()).unwrap_or("".to_owned()),
                )
            })
            .collect();
        terminal::Volumes(volumes)
    }

    use cmd::Body as B;
    Ok(match body {
        B::AddUserCode { id, code } => {
            let mut user = map_err_html(index.user_mut(id))?;
            user.add_code(code.to_lowercase());
            terminal::user(user_info(user.as_immut()))
        }
        B::DeleteEntry { id } => {
            let entry = map_err_html(index.entry_mut(id))?;
            let parent_volume = entry.parent_volume_id().to_owned();
            entry.remove();
            let volume = index.volume(parent_volume).unwrap();
            terminal::volume(volume_info(volume))
        }
        B::DeleteSection { id } => {
            let section = map_err_html(index.section_mut(id))?;
            let parent_entry = section.parent_entry_id().to_owned();
            section.remove();
            let entry = index.entry(parent_entry).unwrap();
            terminal::entry(entry_info(entry))
        }
        B::DeleteVolume { id } => {
            let volume = map_err_html(index.volume_mut(id))?;
            volume.remove();
            terminal::volumes(volumes(&index))
        }
        B::GetContent { id } => {
            let section = map_err_html(index.section(id))?;
            let content = section.content();
            terminal::content(id, content)
        }
        B::GetEntry { id } => {
            let entry = map_err_html(index.entry(id))?;
            terminal::entry(entry_info(entry))
        }
        B::GetIntro { id } => match id {
            None => {
                let content = fs::read_to_string("content/edat.intro").unwrap();
                terminal::content("edat", content)
            }
            Some(id) => {
                let volume = map_err_html(index.volume(id.clone()))?;
                let content = volume.intro();
                terminal::content(id, content)
            }
        },
        B::GetSection { id } => {
            let section = map_err_html(index.section(id))?;
            terminal::section(section_info(section))
        }
        B::GetUser { id } => {
            let user = map_err_html(index.user(id))?;
            terminal::user(user_info(user))
        }
        B::GetVolume { id } => {
            let volume = map_err_html(index.volume(id))?;
            terminal::volume(volume_info(volume))
        }
        B::Images => terminal::images(),
        B::InitUser { id } => {
            let mut user = map_err_html(index.user_mut(id))?;
            user.init();
            terminal::user(user_info(user.as_immut()))
        }
        B::MoveEntry { id, position } => {
            let mut entry = map_err_html(index.entry_mut(id))?;
            map_err_html(entry.move_to(position))?;
            terminal::entry(entry_info(entry.as_immut()))
        }
        B::MoveSection { id, position } => {
            let mut section = map_err_html(index.section_mut(id))?;
            map_err_html(section.move_to(position))?;
            terminal::section(section_info(section.as_immut()))
        }
        B::MoveVolume { id, position } => {
            let mut volume = map_err_html(index.volume_mut(id))?;
            map_err_html(volume.move_to(position))?;
            terminal::volume(volume_info(volume.as_immut()))
        }
        B::NextSectionId => return Ok(Json(index.next_section_id()).into_response()),
        B::NewEntry => terminal::edit_entry(None),
        B::NewSection { date } => terminal::edit_section(None, &date),
        B::NewUser => terminal::edit_user(None),
        B::NewVolume => terminal::edit_volume(None),
        B::RemoveUserCode { id, code } => {
            let mut user = map_err_html(index.user_mut(id))?;
            user.remove_code(&code.to_lowercase());
            terminal::user(user_info(user.as_immut()))
        }
        B::SectionStatus { id, status } => {
            let mut section = map_err_html(index.section_mut(id))?;
            section.set_status(status);
            terminal::section(section_info(section.as_immut()))
        }
        B::SetContent { id, content } => {
            let mut section = map_err_html(index.section_mut(id))?;
            section.set_content(&content);
            terminal::section(section_info(section.as_immut()))
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
            terminal::entry(entry_info(entry.as_immut()))
        }
        B::SetIntro { id, content } => {
            if let Some(id) = id {
                let mut volume = map_err_html(index.volume_mut(id))?;
                volume.set_intro(&content);
                terminal::volume(volume_info(volume.as_immut()))
            } else {
                fs::write("content/edat.intro", &content).unwrap();
                terminal::content("edat", content)
            }
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
            terminal::entry(entry_info(entry.as_immut()))
        }
        B::SetNewSection {
            position,
            heading,
            description,
            summary,
            date,
        } => {
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|_| terminal::bad_date(&date))?;
            let section = map_err_html(index.create_section(
                (!heading.is_empty()).then_some(&heading),
                &description,
                &summary,
                date,
                position,
            ))?;
            terminal::section(section_info(section.as_immut()))
        }
        B::SetNewUser {
            first_name,
            last_name,
        } => {
            let user = map_err_html(index.create_user(first_name, last_name))?;
            terminal::user(user_info(user.as_immut()))
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
            terminal::volume(volume_info(volume.as_immut()))
        }
        B::SetSection {
            id,
            heading,
            description,
            summary,
            date,
        } => {
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|_| terminal::bad_date(&date))?;
            let mut section = map_err_html(index.section_mut(id))?;
            section.set_heading((!heading.is_empty()).then_some(&heading));
            section.set_description(&description);
            section.set_summary(&summary);
            section.set_date(date);
            terminal::section(section_info(section.as_immut()))
        }
        B::SetUser {
            id,
            first_name,
            last_name,
        } => {
            let mut user = map_err_html(index.user_mut(id))?;
            map_err_html(user.set_name(first_name, last_name))?;
            terminal::user(user_info(user.as_immut()))
        }
        B::SetVolume {
            id,
            title,
            subtitle,
        } => {
            let mut volume = map_err_html(index.volume_mut(id))?;
            map_err_html(volume.set_title(&title))?;
            volume.set_subtitle((!subtitle.is_empty()).then_some(&subtitle));
            terminal::volume(volume_info(volume.as_immut()))
        }
        B::UserPrivilege { id, privilege } => {
            let mut user = map_err_html(index.user_mut(id))?;
            user.set_privilege(privilege);
            terminal::user(user_info(user.as_immut()))
        }
        B::VolumeContentType { id, kind } => {
            let mut volume = map_err_html(index.volume_mut(id))?;
            volume.set_kind(kind);
            terminal::volume(volume_info(volume.as_immut()))
        }
        B::Volumes => terminal::volumes(volumes(&index)),
    }
    .into_response())
}
