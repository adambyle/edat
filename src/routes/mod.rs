use axum::body::Bytes;
use axum::extract::{Path as ReqPath, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use maud::Markup;

use crate::data::{user::Privilege as UserPrivilege, *};
use crate::html;
use crate::AppState;

pub mod auth;
pub mod cmd;
pub mod components;
pub mod files;
pub mod pages;
pub mod user;

pub fn get_cookie<'a>(headers: &'a HeaderMap, key: &str) -> Option<&'a str> {
    let cookie = headers.get("Cookie")?.to_str().unwrap();
    cookie.split(&format!("{key}=")).nth(1)?.split(';').next()
}
