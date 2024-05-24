use std::io::Read;
use std::{fs::File, path::Path};

use axum::extract::{Path as ReqPath, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::Value;

use crate::{data, html};

pub async fn script(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file("scripts", file_name, "text/javascript")
}

pub async fn style(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file("styles", file_name, "text/css")
}

pub async fn login(
    State(mut index): State<data::Index>,
    ReqPath((name, code)): ReqPath<(String, String)>,
) -> impl IntoResponse {
    let name = name.to_lowercase().replace(char::is_whitespace, "");
    let code = code.to_lowercase();

    for user in index.users() {
        if (name == user.first_name().to_lowercase() || name == user.id()) && user.has_code(&code) {
            return (StatusCode::OK, user.id().to_owned());
        }
    }

    (StatusCode::UNAUTHORIZED, "".to_owned())
}

pub async fn register(
    State(index): State<data::Index>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
}

pub async fn home(headers: HeaderMap, State(mut index): State<data::Index>) -> impl IntoResponse {
    let user = match login_check(&headers, &mut index) {
        Ok(user) => user,
        Err(html) => return html,
    };
    let Some(history) = user.history() else {
        drop(user);
        return html::setup(&headers, &mut index);
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

fn login_check<'a>(
    headers: &HeaderMap,
    index: &'a mut data::Index,
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
