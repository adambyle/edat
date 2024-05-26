use std::collections::HashMap;
use std::io::Read;
use std::{fs::File, path::Path};

use axum::extract::{Path as ReqPath, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

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
            let read = history.iter().any(|h| h.section() == *section.id());
            let parent_volume = parent_entry.parent_volume(&index);
            let parent_volume = (
                parent_volume.title().to_owned(),
                parent_entry.parent_volume_part(),
                parent_volume.volume_count(),
            );

            recents.push(html::home::RecentSection {
                id: *section.id(),
                parent_volume,
                parent_entry: parent_entry.title().to_owned(),
                in_entry: (in_entry + 1, parent_entry.section_ids().len()),
                date: data::date(section.date()),
                previous: following,
                description: section.description().to_owned(),
                summary: section.summary().to_owned(),
                length: section.length_str(),
                read,
            });
        }
        let expand = user
            .preferences()
            .get("expand_recents");
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

    html::home(headers, widgets)
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
