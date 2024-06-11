use axum::http::HeaderMap;
use maud::{html, Markup, PreEscaped};

use crate::routes::get_cookie;
use crate::data::*;

pub mod cmd;
pub mod pages;
pub mod widgets;
mod wrappers;

pub mod profile {
    use maud::{html, Markup, PreEscaped};
}
