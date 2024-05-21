use std::io::Read;
use std::{fs::File, path::Path};

use axum::extract::{Path as ReqPath, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::Value;

use crate::data::User;
use crate::{html, AppState};

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
    let name = name.to_lowercase().replace(' ', "");
    let code = code.to_lowercase();

    for user in &state.users {
        if (name == user.first_name.to_lowercase() || name == user.full_name())
            && user.codes.contains(&code)
        {
            return (StatusCode::OK, user.full_name());
        }
    }

    (StatusCode::UNAUTHORIZED, String::new())
}

pub async fn record(
    State(state): State<AppState>, Json(read): Json<Value>
) -> impl IntoResponse {
    StatusCode::OK
}

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> impl IntoResponse {
    let user = match login_check(&headers, &state) {
        Ok(user) => user,
        Err(html) => return html,
    };
    let sections_read = match &user.sections_read {
        Some(sections_read) => sections_read,
        None => return html::setup(&headers, &state.content),
    };
    maud::html! { "Success" }
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

fn login_check<'a>(headers: &HeaderMap, state: &'a AppState) -> Result<&'a User, maud::Markup> {
    let err = || html::login(headers);

    let Some(username) = get_cookie(headers, "edat_user") else {
        return Err(err());
    };

    state.users.iter().find(|u| u.full_name() == username).ok_or_else(err)
}

pub fn get_cookie<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    let cookie = headers.get("Cookie")?.to_str().unwrap();
    cookie.split(&format!("{key}=")).nth(1)?.split(';').next()
}
