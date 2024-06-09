#![allow(unused)]

use comments::CommentData;
pub use comments::{Comment, Thread};
use entry::EntryData;
pub use entry::{Entry, EntryMut};
use history::HistoryEntry;
pub use history::{EntryProgress, LineProgress, SectionProgress};
pub use index::Index;
use regex::Regex;
use section::SectionData;
pub use section::{Section, SectionMut};
use serde::Deserialize;
use user::UserData;
pub use user::{User, UserMut};
use volume::VolumeData;
pub use volume::{Volume, VolumeMut};

pub mod comments;
pub mod entry;
pub mod history;
pub mod index;
pub mod section;
pub mod user;
pub mod volume;

#[derive(Debug)]
pub struct DuplicateIdError<Id>(pub Id);

#[derive(Deserialize)]
pub enum Position<Parent, Sibling> {
    StartOf(Parent),
    Before(Sibling),
    After(Sibling),
    EndOf(Parent),
}

fn create_id(name: &str) -> String {
    let name: String = name
        .replace("<i>", "")
        .replace("&", "and")
        .replace("</i>", "")
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '-')
        .collect();
    name.replace(' ', "-")
}

fn process_text(text: &str) -> String {
    let text = text
        .replace("--", "—")
        .replace("-.", "–")
        .replace("...", "…");

    let open_quote = Regex::new(r#""\S"#).unwrap();
    let text = open_quote.replace_all(&text, "“");

    let quote = Regex::new(r#"""#).unwrap();
    let text = quote.replace_all(&text, "”");

    let open_single = Regex::new(r#"(\s')|(^')|(["“”]')"#).unwrap();
    let text = open_single.replace_all(&text, "‘");

    let quote = Regex::new(r#"'"#).unwrap();
    let text = quote.replace_all(&text, "’");

    let lines: Vec<_> = text.lines().filter(|&l| !l.is_empty()).collect();
    lines.join("\n")
}
