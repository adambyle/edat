#![allow(unused)]

use std::fmt::Display;

use chrono::{Datelike, NaiveDate, Utc};
use comments::CommentData;
pub use comments::{Comment, Thread};
use entry::EntryData;
pub use entry::{Entry, EntryMut};
use history::HistoryEntry;
pub use history::{EntryProgress, SectionProgress};
pub use index::Index;
use regex::Regex;
use section::SectionData;
pub use section::{Section, SectionMut};
use serde::Deserialize;
use user::UserData;
pub use user::{User, UserMut};
use volume::VolumeData;
pub use volume::{Volume, VolumeMut};
pub use music::{ListenedAlbum, ListenedTrack, MonthInReview, Rating};

/// Data structures for user comments.
pub mod comments;

/// Data structures for entries, which contain sections.
pub mod entry;

/// Data structures for user reading history.
pub mod history;

/// Data structures for the index binidng all the website's resources.
pub mod index;

/// Music review data structures.
pub mod music;

/// Data structures for sections which contain the text content.
pub mod section;

/// Data structures for users.
pub mod user;

/// Data structures for volumes, which contain a series of entries.
pub mod volume;

#[derive(Debug)]
pub enum DataError {
    /// Wraps the id of a resource that already exists.
    DuplicateId(String),

    /// Wraps the resource type and the id of the resource that does not exist.
    MissingResource(&'static str, String),
}

pub type DataResult<T> = Result<T, DataError>;
/// Represents a resource position relative to some other resource.
#[derive(Deserialize)]
pub enum Position<Parent, Sibling> {
    /// Position at the start of the parent resource.
    StartOf(Parent),

    /// Position at the end of the parent resource.
    EndOf(Parent),

    /// Position before the specified resource.
    Before(Sibling),

    /// Position after the specified resource.
    After(Sibling),
}

impl Position<(), String> {
    pub(crate) fn resolve(self, index: &Index) -> DataResult<usize> {
        Ok(match self {
            Position::StartOf(()) => 0,
            Position::EndOf(()) => index.volumes.len(),
            Position::Before(sibling) => {
                let sibling = index.volume(sibling)?;
                sibling.index_in_list()
            }
            Position::After(sibling) => {
                let sibling = index.volume(sibling)?;
                1 + sibling.index_in_list()
            }
        })
    }
}

impl Position<(String, usize), String> {
    pub(crate) fn resolve(self, index: &mut Index) -> DataResult<(VolumeMut, usize, usize)> {
        Ok(match self {
            Position::StartOf((volume, part)) => {
                let volume = index.volume_mut(volume)?;
                (volume, part, 0)
            }
            Position::EndOf((volume, part)) => {
                let volume = index.volume_mut(volume)?;
                let index_in_volume = volume.entry_count();
                (volume, part, index_in_volume)
            }
            Position::Before(sibling) => {
                let sibling = index.entry(sibling)?;
                let index_in_volume = sibling.index_in_parent();
                let volume_part = sibling.parent_volume_part();
                let volume = index
                    .volume_mut(sibling.parent_volume_id().to_owned())
                    .unwrap();
                (volume, volume_part, index_in_volume)
            }
            Position::After(sibling) => {
                let sibling = index.entry(sibling)?;
                let index_in_volume = sibling.index_in_parent();
                let volume_part = sibling.parent_volume_part();
                let volume = index
                    .volume_mut(sibling.parent_volume_id().to_owned())
                    .unwrap();
                (volume, volume_part, 1 + index_in_volume)
            }
        })
    }
}

impl Position<String, u32> {
    pub(crate) fn resolve(self, index: &mut Index) -> DataResult<(EntryMut, usize)> {
        Ok(match self {
            Position::StartOf(entry) => {
                let entry = index.entry_mut(entry)?;
                (entry, 0)
            }
            Position::EndOf(entry) => {
                let entry = index.entry_mut(entry)?;
                let index_in_entry = entry.section_count();
                (entry, index_in_entry)
            }
            Position::Before(sibling) => {
                let sibling = index.section(sibling)?;
                let index_in_entry = sibling.index_in_parent();
                let entry = index
                    .entry_mut(sibling.parent_entry_id().to_owned())
                    .unwrap();
                (entry, index_in_entry)
            }
            Position::After(sibling) => {
                let sibling = index.section(sibling)?;
                let index_in_entry = sibling.index_in_parent();
                let entry = index
                    .entry_mut(sibling.parent_entry_id().to_owned())
                    .unwrap();
                (entry, 1 + index_in_entry)
            }
        })
    }
}

pub fn date_string(date: &NaiveDate) -> String {
    if date.year() == Utc::now().year() {
        date.format("%b %-d").to_string()
    } else {
        date.format("%b %-d, %Y").to_string()
    }
}

pub fn strip_formatting(text: &str) -> String {
    text.replace("<i>", "").replace("</i>", "")
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

    let open_quote = Regex::new(r#""(\S)"#).unwrap();
    let text = open_quote.replace_all(&text, r"“$1");

    let quote = Regex::new(r#"""#).unwrap();
    let text = quote.replace_all(&text, "”");

    let open_single = Regex::new(r"(\s)'").unwrap();
    let text = open_single.replace_all(&text, "$1‘");

    let open_single = Regex::new(r"^'").unwrap();
    let text = open_single.replace_all(&text, "‘");

    let open_single = Regex::new(r#"(["“”])'"#).unwrap();
    let text = open_single.replace_all(&text, "$1‘");

    let quote = Regex::new(r#"'"#).unwrap();
    let text = quote.replace_all(&text, "’");

    let lines: Vec<_> = text.lines().filter(|&l| !l.is_empty()).collect();
    lines.join("\n")
}
