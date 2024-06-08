use std::collections::HashMap;
use std::fmt::Display;
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
use serde::Deserialize;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::html::home::Widget;
use crate::{data, html, AppState};

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

    let history = user.history().unwrap();
    let two_months_ago = Utc::now().timestamp() - 60 * 60 * 24 * 60;
    let mut history: Vec<_> = history.iter().collect();
    history.sort_by(data::history_sorter);
    let sections = history
        .iter()
        .filter(|h| h.timestamp() >= two_months_ago)
        .filter_map(|h| {
            let section = index.section(h.section_id());
            section.map(|s| {
                let entry = index.entry(s.parent_entry_id()).unwrap();
                let index = entry
                    .section_ids()
                    .iter()
                    .position(|id| id == s.id())
                    .expect(&format!(
                        "section {} not found in parent entry {}",
                        s.id(),
                        entry.id()
                    ));
                html::profile::ViewedSection {
                    entry: entry.title().to_owned(),
                    description: s.description().to_owned(),
                    timestamp: h.timestamp(),
                    id: *s.id(),
                    index: (index, entry.section_ids().len()),
                    progress: (h.progress(), s.lines()),
                }
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
    let mut user = index.user_mut(user).unwrap();

    user.set_widgets(widgets);

    StatusCode::OK
}

#[derive(Deserialize)]
pub struct ReadQuery {
    progress: Option<usize>,
    finished: Option<bool>,
}

pub async fn read(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath(id): ReqPath<u32>,
    Query(options): Query<ReadQuery>,
) -> StatusCode {
    let mut index = state.index.lock().unwrap();
    let user = get_cookie(&headers, "edat_user").unwrap();
    let mut user = index.user_mut(user).unwrap();

    user.read_section(id, options.progress, options.finished.unwrap_or(false));

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
        sections.extend(index.sections().map(|s| *s.id()));
    } else {
        // Load the specific entries.
        for entry in body.entries {
            let Some(entry) = index.entry(&entry) else {
                continue;
            };
            sections.extend(entry.section_ids());
        }
    }

    // Update the user's history and widget preferences.
    let user = get_cookie(&headers, "edat_user").unwrap();
    let mut user = index.user_mut(user).unwrap();
    if sections.len() == 0 {
        user.empty_history();
    }
    for section in sections {
        user.read_section(section, None, true);
    }
    user.set_widgets(body.widgets);
}

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let index = state.index.lock().unwrap();
    let user = match login_check(&headers, &index) {
        Ok(user) => user,
        Err(html) => return html,
    };

    // Direct the user to a setup screen if the history is uninitialized.
    let Some(history) = user.history() else {
        // Collect the entries a user may specify they have read
        // (from standard journal volumes).
        let volumes = index
            .volumes()
            .filter(|v| v.content_type() == data::ContentType::Journal)
            .map(|v| {
                let entries = v
                    .entries(&index)
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
    };

    // Initialize homepage widgets.
    let mut widgets = Vec::new();

    let recent_widget = || {
        // Get all the complete sections from journal volumes.
        let mut sections: Vec<_> = index
            .sections()
            .filter(|s| {
                let parent_entry = index.entry(s.parent_entry_id()).unwrap();
                index
                    .volume(parent_entry.parent_volume_id())
                    .unwrap()
                    .content_type()
                    == data::ContentType::Journal
                    && s.status() == data::ContentStatus::Complete
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
                let parent_entry = index.entry(section.parent_entry_id()).unwrap();
                let in_entry = parent_entry
                    .section_ids()
                    .iter()
                    .position(|s| s == section.id())
                    .unwrap();
                let previous = (in_entry > 0).then(|| {
                    let previous_section = parent_entry.section_ids()[in_entry - 1];
                    let previous_description = index
                        .section(previous_section)
                        .unwrap()
                        .description()
                        .to_owned();
                    (previous_section, previous_description)
                });
                let read = history
                    .iter()
                    .find(|h| h.section_id() == *section.id())
                    .map(|h| h.timestamp());
                let parent_volume = index.volume(parent_entry.parent_volume_id()).unwrap();
                let parent_volume = (
                    parent_volume.title().to_owned(),
                    parent_entry.parent_volume_index(),
                    parent_volume.volume_count(),
                );

                html::home::RecentSection {
                    id: *section.id(),
                    parent_volume,
                    parent_entry: parent_entry.title().to_owned(),
                    in_entry: (in_entry + 1, parent_entry.section_ids().len()),
                    date: data::date_naive(&section.date()),
                    previous,
                    description: section.description().to_owned(),
                    summary: section.summary().to_owned(),
                    length: section.length_str(),
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
            .filter(|v| v.content_type() == data::ContentType::Journal)
            .map(|v| html::home::LibraryVolume {
                title: v.title().to_owned(),
                id: v.id().clone(),
                subtitle: v.subtitle().map(|s| s.to_owned()),
                entry_count: v.entry_ids().len(),
            })
            .collect();

        html::home::LibraryWidget { volumes }
    };

    for widget in user.widgets() {
        let widget: Box<dyn Widget> = match widget.as_ref() {
            "recent-widget" => Box::new(recent_widget()),
            "library-widget" => Box::new(library_widget()),
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
    let mut user = index.user_mut(user).unwrap();
    for (k, v) in body {
        match v {
            Some(v) => {
                user.preferences_mut().insert(k, v);
            }
            None => {
                user.preferences_mut().remove(&k);
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

fn login_check<'a>(
    headers: &HeaderMap,
    index: &'a data::Index,
) -> Result<data::UserWrapper<'a>, maud::Markup> {
    let err = || html::login(headers);

    let Some(username) = get_cookie(headers, "edat_user") else {
        return Err(err());
    };

    index.users().find(|u| u.id() == username).ok_or_else(err)
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
    html::terminal(&headers, user.privilege() == data::UserPrivilege::Owner)
}

mod cmd {
    use serde::Deserialize;

    use crate::data;

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
            position: data::Position<String, u32>,
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
            position: data::Position<String, u32>,
        },
        SectionStatus {
            id: u32,
            status: data::ContentStatus,
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
            position: data::Position<(String, usize), String>,
            description: String,
            summary: String,
        },
        DeleteEntry {
            id: String,
        },
        MoveEntry {
            id: String,
            position: data::Position<(String, usize), String>,
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
            position: data::Position<(), String>,
            title: String,
            subtitle: String,
        },
        DeleteVolume {
            id: String,
        },
        MoveVolume {
            id: String,
            position: data::Position<(), String>,
        },
        VolumeContentType {
            id: String,
            content_type: data::ContentType,
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
        DeleteUser {
            id: String,
        },
        UserPrivilege {
            id: String,
            privilege: data::UserPrivilege,
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
        GetContents {
            id: u32,
        },
        SetContents {
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

    fn or_terminal_error<T>(
        option: Option<T>,
        category: &str,
        id: impl Display,
    ) -> Result<T, maud::Markup> {
        option.ok_or_else(|| html::terminal::error(category, id))
    }

    fn validate<T>(position_result: Result<T, data::InvalidReference>) -> Result<T, maud::Markup> {
        use data::InvalidReference as I;
        position_result.map_err(|err| match err {
            I::Section(id) => html::terminal::error("section", id),
            I::Entry(id) => html::terminal::error("entry", id),
            I::Volume(id) => html::terminal::error("volume", id),
        })
    }

    fn user_info(index: &data::Index, user: data::UserWrapper) -> terminal::UserInfo {
        let codes = user.codes().join(" ");

        // Transform the list of read sections into a list of read entries (and their date).
        let mut history: Vec<(String, i64)> = Vec::new();
        if let Some(user_history) = user.history() {
            for user_history_entry in user_history {
                let section = index.section(user_history_entry.section_id()).unwrap();
                let parent_entry = section.parent_entry_id();
                let history_entry = history.iter_mut().find(|h| h.0 == parent_entry);
                match history_entry {
                    Some(history_entry) => {
                        history_entry.1 = history_entry.1.max(user_history_entry.timestamp())
                    }
                    None => history.push((parent_entry.to_owned(), user_history_entry.timestamp())),
                }
            }
        }
        let mut history: Vec<_> = history
            .into_iter()
            .map(|h| terminal::UserHistoryEntry {
                entry: h.0,
                date: h.1,
            })
            .collect();
        history.sort_by(|a, b| b.date.cmp(&a.date));
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

    fn volume_info(index: &data::Index, volume: data::VolumeWrapper) -> terminal::VolumeInfo {
        let entries = volume
            .entries(&index)
            .map(|e| terminal::VolumeEntry {
                id: e.id().to_owned(),
                description: e.description().to_owned(),
            })
            .collect();
        terminal::VolumeInfo {
            id: volume.id().to_owned(),
            title: volume.title().to_owned(),
            subtitle: volume.subtitle().unwrap_or("").to_owned(),
            owner: volume.owner_id().to_owned(),
            content_type: format!("{:?}", volume.content_type()),
            entries,
            volume_count: volume.volume_count(),
        }
    }

    fn entry_info(index: &data::Index, entry: data::EntryWrapper) -> terminal::EntryInfo {
        let sections = entry
            .sections(&index)
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
                entry.parent_volume_index(),
            ),
            sections,
        }
    }

    fn section_info(index: &data::Index, section: data::SectionWrapper) -> terminal::SectionInfo {
        let parent_entry = index.entry(section.parent_entry_id()).unwrap();
        let in_entry = parent_entry
            .section_ids()
            .iter()
            .position(|s| s == section.id())
            .unwrap();
        let perspectives = section
            .perspective_ids()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let comments = section
            .comments()
            .iter()
            .map(|c| terminal::SectionComment {
                author: c.author_id().to_owned(),
                contents: c.contents().to_owned(),
                timestamp: c.timestamp(),
            })
            .collect();
        terminal::SectionInfo {
            id: *section.id(),
            heading: section.heading().unwrap_or("").to_owned(),
            description: section.description().to_owned(),
            summary: section.summary().to_owned(),
            date: section.raw_date().to_owned(),
            parent_entry: section.parent_entry_id().to_owned(),
            in_entry: (in_entry, parent_entry.section_ids().len()),
            length: section.length(),
            status: format!("{:?}", section.status()),
            perspectives,
            comments,
        }
    }

    fn volumes(index: &data::Index) -> terminal::Volumes {
        let volumes = index
            .volumes()
            .map(|v| (v.id().clone(), v.subtitle().unwrap_or("").to_owned()))
            .collect();
        terminal::Volumes(volumes)
    }

    use cmd::Body as B;
    Ok(match body {
        B::AddUserCode { id, code } => {
            // Change the user's codes.
            {
                let user = index.user_mut(&id);
                let mut user = or_terminal_error(user, "user", &id)?;
                user.add_code(code.to_lowercase());
            }

            // Then send the changed data.
            let user = index.user(&id).unwrap();
            terminal::user(user_info(&index, user))
        }
        B::DeleteEntry { id } => {
            // Dump entry data.
            let entry = index.entry(&id);
            let entry = or_terminal_error(entry, "entry", &id)?;
            let now = Utc::now().timestamp();
            let mut dump = File::create(format!("content/deleted/{}-{}", id, now)).unwrap();
            let mut data = File::open(format!("content/entries/{}.json", id)).unwrap();
            let mut data_contents = String::new();
            data.read_to_string(&mut data_contents).unwrap();
            dump.write_all(data_contents.as_bytes()).unwrap();

            // Remove entry from index and parent volume.
            let parent_volume_id = entry.parent_volume_id().to_owned();
            index.remove_entry(&id);
            let parent_volume = index.volume(&parent_volume_id);
            let parent_volume = or_terminal_error(parent_volume, "volume", parent_volume_id)?;
            terminal::volume(volume_info(&index, parent_volume))
        }
        B::DeleteSection { id } => {
            // Dump section contents.
            let section = index.section(id);
            let section = or_terminal_error(section, "section", id)?;
            let now = Utc::now().timestamp();
            let mut dump = File::create(format!("content/deleted/{}-{}", id, now)).unwrap();
            let mut data = File::open(format!("content/sections/{}.json", id)).unwrap();
            let mut data_contents = String::new();
            data.read_to_string(&mut data_contents).unwrap();
            let mut text = File::open(format!("content/sections/{}.txt", id)).unwrap();
            let mut text_contents = String::new();
            text.read_to_string(&mut text_contents).unwrap();
            dump.write_all(data_contents.as_bytes()).unwrap();
            dump.write_all(text_contents.as_bytes()).unwrap();

            // Remove section from index and parent volume.
            let parent_entry_id = section.parent_entry_id().to_owned();
            index.remove_section(id);
            let parent_entry = index.entry(&parent_entry_id);
            let parent_entry = or_terminal_error(parent_entry, "entry", parent_entry_id)?;
            terminal::entry(entry_info(&index, parent_entry))
        }
        B::DeleteUser { id } => {
            // "Removing the user" just deletes their codes.
            let user = index.user(&id);
            or_terminal_error(user, "user", &id)?;
            index.remove_user(&id);

            // Then send the changed data.
            let user = index.user(&id).unwrap();
            terminal::user(user_info(&index, user))
        }
        B::DeleteVolume { id } => {
            // Dump volume data.
            let volume = index.volume(&id);
            or_terminal_error(volume, "volume", &id)?;
            let now = Utc::now().timestamp();
            let mut dump = File::create(format!("content/deleted/{}-{}", id, now)).unwrap();
            let mut data = File::open(format!("content/volumes/{}.json", id)).unwrap();
            let mut data_contents = String::new();
            data.read_to_string(&mut data_contents).unwrap();
            dump.write_all(data_contents.as_bytes()).unwrap();

            // Remove volume from index.
            index.remove_volume(&id);
            terminal::volumes(volumes(&index))
        }
        B::GetContents { id } => {
            let section = index.section(id);
            or_terminal_error(section, "section", &id)?;
            let mut section_file = File::open(format!("content/sections/{}.txt", id)).unwrap();
            let mut contents = String::new();
            section_file.read_to_string(&mut contents).unwrap();
            terminal::contents(id, contents)
        }
        B::GetEntry { id } => {
            let entry = index.entry(&id);
            let entry = or_terminal_error(entry, "entry", &id)?;
            terminal::entry(entry_info(&index, entry))
        }
        B::GetIntro { id } => match id {
            None => {
                let mut intro_file = File::open("content/edat.intro").unwrap();
                let mut contents = String::new();
                intro_file.read_to_string(&mut contents).unwrap();
                terminal::contents("edat", contents)
            }
            Some(id) => {
                let volume = index.volume(&id);
                or_terminal_error(volume, "volume", &id)?;
                let mut intro_file = File::open(format!("content/volumes/{}.intro", id)).unwrap();
                let mut contents = String::new();
                intro_file.read_to_string(&mut contents).unwrap();
                terminal::contents(id, contents)
            }
        },
        B::GetSection { id } => {
            let section = index.section(id);
            let section = or_terminal_error(section, "section", &id)?;
            terminal::section(section_info(&index, section))
        }
        B::GetUser { id } => {
            let user = index.user(&id);
            let user = or_terminal_error(user, "user", &id)?;
            terminal::user(user_info(&index, user))
        }
        B::GetVolume { id } => {
            let volume = index.volume(&id);
            let volume = or_terminal_error(volume, "volume", &id)?;
            terminal::volume(volume_info(&index, volume))
        }
        B::Images => terminal::images(),
        B::MoveEntry { id, position } => {
            validate(index.move_entry(&id, position))?;
            let entry = index.entry(&id).unwrap();
            terminal::entry(entry_info(&index, entry))
        }
        B::MoveSection { id, position } => {
            validate(index.move_section(id, position))?;
            let section = index.section(id).unwrap();
            terminal::section(section_info(&index, section))
        }
        B::MoveVolume { id, position } => {
            validate(index.move_volume(&id, position))?;
            let volume = index.volume(&id).unwrap();
            terminal::volume(volume_info(&index, volume))
        }
        B::NextSectionId => return Ok(Json(index.next_section_id()).into_response()),
        B::NewEntry => terminal::edit_entry(None),
        B::NewSection { date } => terminal::edit_section(None, &date),
        B::NewUser => terminal::edit_user(None),
        B::NewVolume => terminal::edit_volume(None),
        B::RemoveUserCode { id, code } => {
            {
                let user = index.user_mut(&id);
                let mut user = or_terminal_error(user, "user", &id)?;
                user.remove_code(&code.to_lowercase());
            }
            let user = index.user(&id).unwrap();
            terminal::user(user_info(&index, user))
        }
        B::SectionStatus { id, status } => {
            {
                let section = index.section_mut(id);
                let mut section = or_terminal_error(section, "section", id)?;
                section.set_status(status);
            }
            let section = index.section(id).unwrap();
            terminal::section(section_info(&index, section))
        }
        B::SetContents {
            id,
            content: contents,
        } => {
            // Check section exists.
            let section = index.section_mut(id);
            let section = or_terminal_error(section, "section", &id)?;

            let contents = data::process_text(&contents);
            let mut contents_file = File::create(format!("content/sections/{}.txt", id)).unwrap();
            contents_file.write_all(contents.as_bytes()).unwrap();

            // Force a save.
            drop(section);

            terminal::contents(id, contents)
        }
        B::SetEntry {
            id,
            title,
            description,
            summary,
        } => {
            let id = validate(index.set_entry_title(&id, title))?;
            {
                let entry = index.entry_mut(&id);
                let mut entry = or_terminal_error(entry, "entry", &id)?;
                entry.set_description(description);
                entry.set_summary(summary);
            }
            let entry = index.entry(&id).unwrap();
            terminal::entry(entry_info(&index, entry))
        }
        B::SetIntro {
            id,
            content: contents,
        } => {
            let (filename, volume) = match id {
                None => ("content/edat.intro".to_owned(), None),
                Some(ref id) => {
                    // Verify volume exists.
                    let volume = index.volume_mut(id);
                    let volume = or_terminal_error(volume, "volume", id)?;

                    (format!("content/volumes/{id}.intro"), Some(volume))
                }
            };
            let contents = data::process_text(&contents);
            let mut intro_file = File::create(filename).unwrap();
            intro_file.write_all(contents.as_bytes()).unwrap();

            // Force a save.
            if let Some(volume) = volume {
                // Force save.
                drop(volume);
            }

            terminal::contents(id.unwrap_or("edat".to_owned()), contents)
        }
        B::SetNewEntry {
            title,
            position,
            description,
            summary,
        } => {
            let id =
                validate(index.new_entry(title, description, summary, user.to_owned(), position))?;
            let entry = index.entry(&id).unwrap();
            terminal::entry(entry_info(&index, entry))
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
            let heading = if heading == "" { None } else { Some(heading) };
            let id = validate(index.new_section(heading, description, summary, date, position))?;
            let section = index.section(id).unwrap();
            terminal::section(section_info(&index, section))
        }
        B::SetNewUser {
            first_name,
            last_name,
        } => {
            index.new_user(first_name.clone(), last_name.clone());
            let user_id = format!("{}{}", first_name.to_lowercase(), last_name.to_lowercase());
            let user = index.user(&user_id).unwrap();
            terminal::user(user_info(&index, user))
        }
        B::SetNewVolume {
            position,
            title,
            subtitle,
        } => {
            let subtitle = if subtitle == "" { None } else { Some(subtitle) };
            let id = validate(index.new_volume(title, subtitle, user.to_owned(), position))?;
            let volume = index.volume(&id).unwrap();
            terminal::volume(volume_info(&index, volume))
        }
        B::SetSection {
            id,
            heading,
            description,
            summary,
            date,
        } => {
            {
                let heading = if heading == "" { None } else { Some(heading) };
                let section = index.section_mut(id);
                let mut section = or_terminal_error(section, "section", &id)?;
                section.set_heading(heading);
                section.set_description(description);
                section.set_summary(summary);
                let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                    .map_err(|_| terminal::bad_date(&date))?;
                section.set_date(date);
            }
            let section = index.section(id).unwrap();
            terminal::section(section_info(&index, section))
        }
        B::SetUser {
            id,
            first_name,
            last_name,
        } => {
            let new_id = index.set_user_name(&id, first_name, last_name);
            let new_id = or_terminal_error(new_id, "user", id)?;
            let user = index.user(&new_id).unwrap();
            terminal::user(user_info(&index, user))
        }
        B::SetVolume {
            id,
            title,
            subtitle,
        } => {
            let id = validate(index.set_volume_title(&id, title))?;
            let subtitle = if subtitle == "" { None } else { Some(subtitle) };
            {
                let volume = index.volume_mut(&id);
                let mut volume = or_terminal_error(volume, "volume", &id)?;
                volume.set_subtitle(subtitle);
            }

            let volume = index.volume(&id).unwrap();
            terminal::volume(volume_info(&index, volume))
        }
        B::UserPrivilege { id, privilege } => {
            {
                let user = index.user_mut(&id);
                let mut user = or_terminal_error(user, "user", &id)?;
                user.set_privilege(privilege);
            }
            let user = index.user(&id).unwrap();
            terminal::user(user_info(&index, user))
        }
        B::VolumeContentType { id, content_type } => {
            {
                let volume = index.volume_mut(&id);
                let mut volume = or_terminal_error(volume, "volume", &id)?;
                volume.set_content_type(content_type);
            }
            let volume = index.volume(&id).unwrap();
            terminal::volume(volume_info(&index, volume))
        }
        B::Volumes => {
            let volumes = index
                .volumes()
                .map(|v| (v.id().to_owned(), v.subtitle().unwrap_or("").to_owned()))
                .collect();
            let volumes = terminal::Volumes(volumes);
            terminal::volumes(volumes)
        }
    }
    .into_response())
}
