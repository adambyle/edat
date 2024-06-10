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

/// Data structures for user comments.
pub mod comments;

/// Data structures for entries, which contain sections.
pub mod entry;

/// Data structures for user reading history.
pub mod history;

/// Data structures for the index binidng all the website's resources.
pub mod index;

/// Data structures for sections which contain the text content.
pub mod section;

/// Data structures for users.
pub mod user;

/// Data structures for volumes, which contain a series of entries.
pub mod volume;

/// A resource was renamed or created with an id that already exists.
#[derive(Debug)]
pub struct DuplicateIdError<Id>(pub Id);

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

impl Position<(), Volume<'_>> {
    pub(crate) fn resolve(self, index: &Index) -> usize {
        match self {
            Position::StartOf(()) => 0,
            Position::EndOf(()) => index.volumes.len(),
            Position::Before(sibling) => {
                index.volumes.keys().position(|v| v == &sibling.id).unwrap()
            }
            Position::After(sibling) => {
                1 + index.volumes.keys().position(|v| v == &sibling.id).unwrap()
            }
        }
    }
}

impl Position<(Volume<'_>, usize), Entry<'_>> {
    pub(crate) fn resolve(self, index: &mut Index) -> (VolumeMut, usize, usize) {
        match self {
            Position::StartOf((volume, part)) => {
                let volume = index.volume_mut(volume.id).unwrap();
                (volume, part, 0)
            }
            Position::EndOf((volume, part)) => {
                let index_in_volume = volume.entry_count();
                let volume = index.volume_mut(volume.id).unwrap();
                (volume, part, index_in_volume)
            }
            Position::Before(sibling) => {
                let index_in_volume = sibling.index_in_parent();
                let volume = index
                    .volume_mut(sibling.parent_volume_id().to_owned())
                    .unwrap();
                (volume, sibling.parent_volume_part(), index_in_volume)
            }
            Position::After(sibling) => {
                let index_in_volume = sibling.index_in_parent();
                let volume = index
                    .volume_mut(sibling.parent_volume_id().to_owned())
                    .unwrap();
                (volume, sibling.parent_volume_part(), 1 + index_in_volume)
            }
        }
    }
}

impl Position<Entry<'_>, Section<'_>> {
    pub(crate) fn resolve(self, index: &mut Index) -> (EntryMut, usize) {
        match self {
            Position::StartOf(entry) => {
                let entry = index.entry_mut(entry.id).unwrap();
                (entry, 0)
            }
            Position::EndOf(entry) => {
                let index_in_entry = entry.section_count();
                let entry = index.entry_mut(entry.id).unwrap();
                (entry, index_in_entry)
            }
            Position::Before(sibling) => {
                let index_in_entry = sibling.index_in_parent();
                let entry = index
                    .entry_mut(sibling.parent_entry_id().to_owned())
                    .unwrap();
                (entry, index_in_entry)
            }
            Position::After(sibling) => {
                let index_in_entry = sibling.index_in_parent();
                let entry = index
                    .entry_mut(sibling.parent_entry_id().to_owned())
                    .unwrap();
                (entry, 1 + index_in_entry)
            }
        }
    }
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
