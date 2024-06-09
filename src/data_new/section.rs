use std::fs::{self, File};

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::search;

use super::*;

#[derive(Serialize, Deserialize)]
pub(super) struct SectionData {
    pub(super) heading: Option<String>,
    pub(super) description: String,
    pub(super) summary: String,
    pub(super) status: Status,
    pub(super) date: String,
    pub(super) comments: Vec<CommentData>,
    pub(super) parent_entry: String,
    pub(super) length: usize,
    pub(super) lines: usize,
    pub(super) perspectives: Vec<u32>,

    #[serde(skip)]
    pub(super) search_index: search::Index,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Status {
    Missing,
    Incomplete,
    Complete,
}

pub struct Section<'index> {
    pub(super) index: &'index Index,
    pub(super) id: u32,
}

macro_rules! immut_fns {
    () => {
        pub(super) fn data(&self) -> &SectionData {
            self.index.sections.get(&self.id).unwrap()
        }

        pub fn id(&self) -> u32 {
            self.id
        }

        pub fn heading(&self) -> Option<&String> {
            self.data().heading.as_ref()
        }

        pub fn description(&self) -> &str {
            &self.data().description
        }

        pub fn summary(&self) -> &str {
            &self.data().summary
        }

        pub fn status(&self) -> Status {
            self.data().status.clone()
        }

        pub fn date(&self) -> NaiveDate {
            NaiveDate::parse_from_str(&self.data().date, "%Y-%m-%d").unwrap()
        }

        pub fn comments(&self, line: usize) -> Thread {
            // Wrap a comment's data.
            fn to_comment<'index>(
                index: &'index Index,
                comment: &'index CommentData,
            ) -> Comment<'index> {
                Comment {
                    uuid: comment.uuid,
                    content: &comment.content,
                    show: comment.show,
                    author: index.user(comment.author.clone()).unwrap(),
                    timestamp: comment.timestamp,
                }
            };

            // The first displaced comment, if any, will have the context.
            let context = self
                .data()
                .comments
                .iter()
                .find(|c| c.line == line && c.displaced)
                .and_then(|c| c.context.clone());

            // Collect and process comments.
            let comments = self
                .data()
                .comments
                .iter()
                .filter(|c| c.line == line && !c.displaced)
                .map(|c| to_comment(&self.index, c))
                .collect();
            let displaced: Vec<_> = self
                .data()
                .comments
                .iter()
                .filter(|c| c.line == line && c.displaced)
                .map(|c| to_comment(&self.index, c))
                .collect();

            Thread {
                line,
                comments,
                displaced,
                context,
            }
        }

        pub fn parent_entry_id(&self) -> &str {
            &self.data().parent_entry
        }

        pub fn parent_entry(&self) -> Entry {
            self.index
                .entry(self.data().parent_entry.to_owned())
                .unwrap()
        }

        pub fn index_in_parent(&self) -> usize {
            self.parent_entry()
                .section_ids()
                .iter()
                .position(|&s| s == self.id)
                .unwrap()
        }

        pub fn length(&self) -> usize {
            self.data().length
        }

        pub fn lines(&self) -> usize {
            self.data().length
        }

        pub fn content(&self) -> String {
            fs::read_to_string(format!("content/sections/{}.txt", self.id)).unwrap()
        }

        pub fn search_index(&self) -> &search::Index {
            &self.data().search_index
        }
    };
}

impl Section<'_> {
    immut_fns!();
}

pub struct SectionMut<'index> {
    pub(super) index: &'index mut Index,
    pub(super) id: u32,
    pub(super) exists: bool,
}

impl SectionMut<'_> {
    pub(super) fn data_mut(&mut self) -> &mut SectionData {
        self.index.sections.get_mut(&self.id).unwrap()
    }

    immut_fns!();

    pub fn set_heading(&mut self, heading: Option<String>) {
        self.data_mut().heading = heading;
    }

    pub fn set_description(&mut self, description: String) {
        self.data_mut().description = description;
    }

    pub fn set_summary(&mut self, summary: String) {
        self.data_mut().description = summary;
    }

    pub fn set_status(&mut self, status: Status) {
        self.data_mut().status = status;
    }

    pub fn set_date(&mut self, date: NaiveDate) {
        self.data_mut().date = date.format("%Y-%m-%d").to_string();
    }

    pub fn add_comment(&mut self, user: User, line: usize, content: String) {
        self.data_mut().comments.push(CommentData {
            uuid: todo!(),
            content: vec![content],
            show: true,
            displaced: false,
            line,
            context: None,
            author: user.id.clone(),
            timestamp: Utc::now().timestamp(),
        });
    }

    pub fn edit_comment(&mut self, uuid: u128, content: String) {
        for comment in &mut self.data_mut().comments {
            if comment.uuid == uuid {
                comment.content.push(content);
                return;
            }
        }
    }

    pub fn remove_comment(&mut self, uuid: u128) {
        for comment in &mut self.data_mut().comments {
            if comment.uuid == uuid {
                comment.show = false;
                return;
            }
        }
    }

    pub fn restore_comment(&mut self, uuid: u128) {
        for comment in &mut self.data_mut().comments {
            if comment.uuid == uuid {
                comment.show = true;
                return;
            }
        }
    }

    pub fn set_context(&mut self, line: usize, context: String) {
        let context = (!context.is_empty()).then_some(context);

        // The first displaced comment, if any, will have the context.
        let Some(comment) = self
            .data_mut()
            .comments
            .iter_mut()
            .find(|c| c.line == line && c.displaced)
        else {
            return;
        };

        comment.context = context;
    }

    pub fn parent_entry_mut(&mut self) -> EntryMut {
        self.index
            .entry_mut(self.data().parent_entry.to_owned())
            .unwrap()
    }

    pub fn set_content(&mut self, content: String) -> Vec<u128> {
        // Format and write the content.
        let content = process_text(&content);
        fs::write(format!("content/sections/{}.txt", self.id), content);

        // TODO update comments and return UUIDs for comments that might need context.

        todo!()
    }

    pub fn move_to(&mut self, position: Position<Entry, Section>) {
        // Detach from current entry.
        let id = self.id;
        self.parent_entry_mut()
            .data_mut()
            .sections
            .retain(|&s| s != id);

        // Get new parent.
        let (mut new_parent_entry, new_index_in_parent) = match position {
            Position::StartOf(entry) => {
                let entry = self.index.entry_mut(entry.id).unwrap();
                (entry, 0)
            }
            Position::EndOf(entry) => {
                let index = entry.section_count();
                let entry = self.index.entry_mut(entry.id).unwrap();
                (entry, index)
            }
            Position::Before(sibling) => {
                let index = sibling.index_in_parent();
                let entry = self
                    .index
                    .entry_mut(sibling.parent_entry_id().to_owned())
                    .unwrap();
                (entry, index)
            }
            Position::After(sibling) => {
                let index = sibling.index_in_parent();
                let entry = self
                    .index
                    .entry_mut(sibling.parent_entry_id().to_owned())
                    .unwrap();
                (entry, 1 + index)
            }
        };

        // Insert section.
        new_parent_entry
            .data_mut()
            .sections
            .insert(new_index_in_parent, self.id.clone());
    }

    pub fn remove(mut self) {
        // Update parent entry.
        let id = self.id;
        self.parent_entry_mut()
            .data_mut()
            .sections
            .retain(|&s| s != id);

        // Update user reading history.
        for user in self.index.users.values_mut() {
            user.history.retain(|h| h.section != id);
        }

        // Update index registry.
        self.index.sections.remove(&self.id);

        // Archive files.
        let now = Utc::now().timestamp();
        fs::rename(
            format!("content/section/{}.json", &self.id),
            format!("/archive/section-{}-{now}", &self.id),
        );
        fs::rename(
            format!("content/section/{}.txt", &self.id),
            format!("/archivecontent-{}-{now}", &self.id),
        );

        // Prevent saving on drop.
        self.exists = false;
    }
}

impl Drop for SectionMut<'_> {
    fn drop(&mut self) {
        if !self.exists {
            return;
        }

        // Create search index.
        let id = self.id.clone();
        let heading = self.heading().map(|h| h.clone()).unwrap_or(String::new());
        let description = self.description().to_owned();
        let summary = self.summary().to_owned();
        let content = self.content();
        let search_index = &mut self.data_mut().search_index;
        *search_index = search::Index::new();
        search_index.add_section("HEADING".to_owned(), &heading);
        search_index.add_section("DESCRIPTION".to_owned(), &description);
        search_index.add_section("SUMMARY".to_owned(), &summary);
        search_index.add_section("CONTENT".to_owned(), &content);
        fs::write(
            format!("content/sections/{id}.index"),
            search_index.to_bytes(),
        )
        .unwrap();

        // Write data.
        let section = File::create(format!("content/sections/{id}.json")).unwrap();
        serde_json::to_writer_pretty(section, self.data()).unwrap();
    }
}
