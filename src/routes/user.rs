use std::collections::HashMap;

use serde::Deserialize;

use super::*;

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

pub async fn set_widgets(
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

pub async fn set_preferences(
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

#[derive(Deserialize)]
pub struct ReadQuery {
    progress: Option<usize>,
    entry: Option<bool>,
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

    if options.entry.unwrap_or(false) {
        println!("Entry finished!");
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
