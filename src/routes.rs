use std::io::Read;
use std::{fs::File, path::Path};

use axum::extract::{Path as ReqPath, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};

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
        if (name == user.first_name.to_lowercase() || name == user.full_name()) && user.codes.contains(&code) {
            return (StatusCode::OK, user.full_name());
        }
    }

    (StatusCode::UNAUTHORIZED, String::new())
}

pub async fn home(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if let Err(login) = login_check(&headers, &state) {
        return login;
    }
    Html("Success".to_string())
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

fn login_check(headers: &HeaderMap, state: &AppState) -> Result<(), Html<String>> {
    let err = || Err(Html(html::login()));

    let Some(cookie) = headers.get("Cookie") else {
        return err();
    };

    let cookie = cookie.to_str().unwrap();
    if !cookie.contains("edat_user=") {
        return err();
    }

    let username = cookie
        .split("edat_user=")
        .last()
        .unwrap()
        .split(';')
        .next()
        .unwrap();

    if !state.users.iter().any(|user| user.full_name() == username) {
        return err();
    }

    Ok(())
}
