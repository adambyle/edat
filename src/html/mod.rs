use axum::http::HeaderMap;
use maud::{html, Markup, PreEscaped};

use crate::routes::get_cookie;
use crate::data::*;

pub mod cmd;
pub mod components;
pub mod pages;
mod wrappers;
