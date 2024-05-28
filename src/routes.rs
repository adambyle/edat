use std::collections::HashMap;
use std::io::{Read, Write};
use std::{fs::File, path::Path};

use axum::extract::{Path as ReqPath, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::data::AnyWrapper;
use crate::html::home::Widget;
use crate::{data, html, upgrade, AppState};

pub async fn script(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file("scripts", file_name, "text/javascript")
}

pub async fn style(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file("styles", file_name, "text/css")
}

pub async fn login(
    State(state): State<AppState>,
    ReqPath((name, code)): ReqPath<(String, String)>,
) -> impl IntoResponse {
    let index = state.index.lock().unwrap();
    let name = name.to_lowercase().replace(char::is_whitespace, "");
    let code = code.to_lowercase();

    for user in index.users() {
        if (name == user.first_name().to_lowercase() || &name == user.id()) && user.has_code(&code)
        {
            return (StatusCode::OK, user.id().to_owned());
        }
    }

    (StatusCode::UNAUTHORIZED, "".to_owned())
}

#[derive(Deserialize)]
pub struct RegisterBody {
    entries: Vec<String>,
    widgets: Vec<String>,
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
        sections.extend(index.sections().map(|s| *s.id()));
    } else {
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
        user.read_section(section, 0, true);
    }
    user.set_widgets(body.widgets);
}

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let index = state.index.lock().unwrap();
    let user = match login_check(&headers, &index) {
        Ok(user) => user,
        Err(html) => return html,
    };
    let Some(history) = user.history() else {
        // Initialize the entries a user may specify they have read.
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
        let mut sections: Vec<_> = index
            .sections()
            .filter(|s| {
                s.parent_entry(&index).parent_volume(&index).content_type()
                    == data::ContentType::Journal
                    && s.status() == data::ContentStatus::Complete
            })
            .collect();
        sections.sort_by(|a, b| {
            let date_a = a.date();
            let date_b = b.date();
            if date_a == date_b {
                b.id().cmp(&a.id())
            } else {
                date_b.cmp(&date_a)
            }
        });

        let mut recents = Vec::new();
        for section in &sections[..10.min(sections.len())] {
            let parent_entry = section.parent_entry(&index);
            let in_entry = parent_entry
                .section_ids()
                .iter()
                .position(|s| s == section.id())
                .unwrap();
            let following = (in_entry > 0).then(|| {
                let previous_section = parent_entry.section_ids()[in_entry - 1];
                let previous_description = index
                    .section(previous_section)
                    .unwrap()
                    .description()
                    .to_owned();
                (previous_section, previous_description)
            });
            let read = history.iter().any(|h| h.section_id() == *section.id());
            let parent_volume = parent_entry.parent_volume(&index);
            let parent_volume = (
                parent_volume.title().to_owned(),
                parent_entry.parent_volume_index(),
                parent_volume.volume_count(),
            );

            recents.push(html::home::RecentSection {
                id: *section.id(),
                parent_volume,
                parent_entry: parent_entry.title().to_owned(),
                in_entry: (in_entry + 1, parent_entry.section_ids().len()),
                date: data::date_naive(&section.date()),
                previous: following,
                description: section.description().to_owned(),
                summary: section.summary().to_owned(),
                length: section.length_str(),
                read,
            });
        }
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

    for widget in user.widgets() {
        let widget: Box<dyn Widget> = match widget.as_ref() {
            "recent-widget" => Box::new(recent_widget()),
            _ => continue,
        };
        widgets.push(widget);
    }

    html::home(&headers, widgets)
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

fn static_file(
    subfolder: &str,
    file_name: String,
    content_type: &'static str,
) -> impl IntoResponse {
    let path = Path::new("static").join(subfolder).join(file_name);

    match File::open(path.clone()) {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                contents,
            )
        }
        Err(_) => {
            println!("Invalid path: {}", path.to_str().unwrap());
            (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/plain")],
                "".to_owned(),
            )
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
            id: u32,
        },
        SetSection {
            id: u32,
            heading: String,
            description: String,
            summary: String,
        },
        SetNewSection {
            id: u32,
            position: Position<String, u32>,
            heading: String,
            description: String,
            summary: String,
        },
        DeleteSection {
            id: u32,
        },
        MoveSection {
            id: u32,
            position: Position<String, u32>,
        },
        SectionStatus {
            id: String,
            status: data::ContentStatus,
        },
        GetEntry {
            id: String,
        },
        NewEntry {
            id: String,
        },
        SetEntry {
            id: String,
            title: String,
            description: String,
            summary: String,
        },
        SetNewEntry {
            id: String,
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
        NewVolume {
            id: String,
        },
        SetVolume {
            id: String,
            title: String,
            subtitle: String,
        },
        SetNewVolume {
            id: String,
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
            content_type: data::ContentType,
        },
        GetUser {
            id: String,
        },
        SetUser {
            first_name: String,
            last_name: String,
        },
        NewUser {
            first_name: String,
            last_name: String,
        },
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
    }

    #[derive(Deserialize)]
    pub enum Position<C, I> {
        StartOf(C),
        Before(I),
        After(I),
        EndOf(C),
    }
}

pub async fn cmd(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(body): Json<cmd::Body>,
) -> Result<maud::Markup, maud::Markup> {
    use html::terminal;
    let index = state.index.lock().unwrap();

    let get_user = |id| index.user(id).ok_or(terminal::error("user", id));
    let get_volume = |id| index.volume(id).ok_or(terminal::error("volume", id));
    let get_entry = |id| index.entry(id).ok_or(terminal::error("entry", id));
    let get_section = |id| index.section(id).ok_or(terminal::error("section", id));

    let user_info = |user: &dyn AnyWrapper<data::User>| {
        let codes = user.codes().join(" ");
        let mut history: Vec<(String, DateTime<Utc>)> = Vec::new();
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
        let history = history
            .into_iter()
            .map(|h| terminal::UserHistoryEntry {
                entry: h.0,
                date: data::date(&h.1),
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
    };

    let volume_info = |volume: &dyn AnyWrapper<data::Volume>| {
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
    };

    let entry_info = |entry: &dyn AnyWrapper<data::Entry>| {
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
    };

    let section_info = |section: &dyn AnyWrapper<data::Section>| {
        let parent_entry = section.parent_entry(&index);
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
                timestamp: data::date(&c.timestamp()),
            })
            .collect();
        terminal::SectionInfo {
            id: *section.id(),
            heading: section.heading().unwrap_or("").to_owned(),
            description: section.description().to_owned(),
            summary: section.summary().to_owned(),
            date: data::date_naive(&section.date()),
            parent_entry: section.parent_entry_id().to_owned(),
            in_entry: (in_entry, parent_entry.section_ids().len()),
            length: section.length(),
            status: format!("{:?}", section.status()),
            perspectives,
            comments,
        }
    };

    use cmd::Body as B;
    Ok(match body {
        B::AddUserCode { id, code } => {
            let user = get_user(&id)?;
            upgrade!(user, index);
            user.add_code(code);
            html::terminal::user(&user_info(&user))
        }
        B::DeleteEntry { id } => {
            let parent_volume = get_entry(&id)?.parent_volume_id();
            let now = Utc::now().timestamp();
            let dump = File::create(format!("content/deleted/{}-{}", id, now)).unwrap();
            let data = File::open(format!("content/entries/{}.json", id)).unwrap();
            let mut data_contents = String::new();
            data.read_to_string(&mut data_contents);
            dump.write_all(data_contents.as_bytes());
            let parent_volume = get_volume(parent_volume)?;
            index.remove_entry(&id);
            html::terminal::volume(&volume_info(&parent_volume))
        }
        B::DeleteSection { id } => {
            let parent_entry = get_section(id)?.parent_entry_id();
            let now = Utc::now().timestamp();
            let dump = File::create(format!("content/deleted/{}-{}", id, now)).unwrap();
            let data = File::open(format!("content/sections/{}.json", id)).unwrap();
            let mut data_contents = String::new();
            data.read_to_string(&mut data_contents);
            let contents = File::open(format!("content/sections/{}.txt", id)).unwrap();
            let mut contents = String::new();
            data.read_to_string(&mut contents);
            dump.write_all(contents.as_bytes());
            let parent_entry = get_entry(parent_entry)?;
            index.remove_section(id);
            html::terminal::entry(&entry_info(&parent_entry))
        }
    });

    Ok(maud::html!("temp"))
}
