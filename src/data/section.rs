use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
};

use chrono::{NaiveDate, Utc};
use levenshtein::levenshtein;
use rand::Rng;
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

/// Section completion status.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Status {
    /// The section is incomplete. It will display as "coming soon...".
    Missing,

    /// The section is incomplete. The text so far will be displayed,
    /// but readers won't be notified about it until it's done.
    Incomplete,

    /// The section is complete.
    Complete,
}

/// A wrapper around a section.
pub struct Section<'index> {
    pub(super) index: &'index Index,
    pub(super) id: u32,
}

macro_rules! immut_fns {
    () => {
        pub(super) fn data(&self) -> &SectionData {
            self.index.sections.get(&self.id).unwrap()
        }

        /// The section id.
        pub fn id(&self) -> u32 {
            self.id
        }

        /// The section heading, if any.
        pub fn heading(&self) -> Option<&String> {
            self.data().heading.as_ref()
        }

        /// The section description (brief explanation).
        pub fn description(&self) -> &str {
            &self.data().description
        }

        /// The section summary (longer explanation).
        pub fn summary(&self) -> &str {
            &self.data().summary
        }

        /// The section completion status.
        pub fn status(&self) -> Status {
            self.data().status.clone()
        }

        /// The section creation date.
        pub fn date(&self) -> NaiveDate {
            NaiveDate::parse_from_str(&self.data().date, "%Y-%m-%d").unwrap()
        }

        /// The comment thread associated with the specified line.
        ///
        /// This will return an empty thread if the line doesn't have any comments.
        pub fn comments(&self, line: usize) -> Thread {
            // Collect and process comments.
            let comments = self
                .data()
                .comments
                .iter()
                .filter(|c| c.line == line)
                .map(|c| Comment {
                    uuid: c.uuid,
                    content: &c.content,
                    show: c.show,
                    author: self.index.user(c.author.clone()).unwrap(),
                    timestamp: c.timestamp,
                })
                .collect();

            Thread { line, comments }
        }

        /// Get all the threads in this section.
        pub fn threads(&self) -> Vec<Thread> {
            (0..self.lines())
                .map(|l| self.comments(l))
                .filter(|t| !t.comments.is_empty())
                .collect()
        }

        /// The id of the parent entry.
        pub fn parent_entry_id(&self) -> &str {
            &self.data().parent_entry
        }

        /// The index of this section in its parent entry.
        pub fn index_in_parent(&self) -> usize {
            self.parent_entry()
                .section_ids()
                .iter()
                .position(|&s| s == self.id)
                .unwrap()
        }

        /// The length in words of this section's content.
        pub fn length(&self) -> usize {
            self.data().length
        }

        /// The length in words of this section's content, as a string.
        pub fn length_string(&self) -> String {
            let length = self.data().length;
            if length < 2000 {
                (length / 100 * 100).to_string()
            } else {
                format!("{:.1}k", (length as f64 / 1000.0))
            }
        }

        /// The number of lines in this section's content.
        pub fn lines(&self) -> usize {
            self.data().lines
        }

        /// The text content of this section.
        pub fn content(&self) -> String {
            fs::read_to_string(format!("content/sections/{}.txt", self.id)).unwrap()
        }

        /// The ids of the perspectives on this section.
        pub fn perspective_ids(&self) -> &[u32] {
            &self.data().perspectives
        }

        /// Get the search index for this section.
        pub fn search_index(&self) -> &search::Index {
            &self.data().search_index
        }
    };
}

impl Section<'_> {
    immut_fns!();
}

impl<'index> Section<'index> {
    /// The parent index.
    pub fn index(&self) -> &'index Index {
        &self.index
    }

    /// Get a wrapper around the parent entry.
    pub fn parent_entry(&self) -> Entry<'index> {
        self.index
            .entry(self.data().parent_entry.to_owned())
            .unwrap()
    }
}

/// A mutable wrapper around a section.
pub struct SectionMut<'index> {
    pub(super) index: &'index mut Index,
    pub(super) id: u32,
    pub(super) exists: bool,
}

impl SectionMut<'_> {
    pub(super) fn data_mut(&mut self) -> &mut SectionData {
        self.index.sections.get_mut(&self.id).unwrap()
    }

    pub fn as_immut(&self) -> Section {
        Section {
            index: &self.index,
            id: self.id.clone(),
        }
    }

    /// Get a wrapper around the parent entry.
    pub fn parent_entry(&self) -> Entry {
        self.index
            .entry(self.data().parent_entry.to_owned())
            .unwrap()
    }

    immut_fns!();

    /// Set the section heading.
    pub fn set_heading(&mut self, heading: Option<&str>) {
        self.data_mut().heading = heading.map(|h| process_text(&h));
    }

    /// Set the section description (brief explanation).
    pub fn set_description(&mut self, description: &str) {
        self.data_mut().description = process_text(description);
    }

    /// Set the section summary (longer explanation).
    pub fn set_summary(&mut self, summary: &str) {
        self.data_mut().summary = process_text(summary);
    }

    /// Set the section status.
    pub fn set_status(&mut self, status: Status) {
        self.data_mut().status = status;
    }

    /// Set the section creation date.
    pub fn set_date(&mut self, date: NaiveDate) {
        self.data_mut().date = date.format("%Y-%m-%d").to_string();
    }

    /// Add a comment by a user to the thread at the specified line.
    pub fn add_comment(&mut self, user: User, line: usize, content: &str) {
        self.data_mut().comments.push(CommentData {
            uuid: rand::thread_rng().gen(),
            content: vec![process_text(content)],
            show: true,
            line,
            author: user.id.clone(),
            timestamp: Utc::now().timestamp(),
        });
    }

    /// Edit a comment's contents by its UUID.
    ///
    /// The comment's past contents are preserved.
    pub fn edit_comment(&mut self, uuid: u128, content: &str) {
        for comment in &mut self.data_mut().comments {
            if comment.uuid == uuid {
                comment.content.push(process_text(content));
                return;
            }
        }
    }

    /// Remove a comment by its UUID from its thread.
    pub fn remove_comment(&mut self, uuid: u128) {
        for comment in &mut self.data_mut().comments {
            if comment.uuid == uuid {
                comment.show = false;
                return;
            }
        }
    }

    /// Restore a comment by its UUID.
    pub fn restore_comment(&mut self, uuid: u128) {
        for comment in &mut self.data_mut().comments {
            if comment.uuid == uuid {
                comment.show = true;
                return;
            }
        }
    }

    /// Get the parent entry for mutation.
    pub fn parent_entry_mut(&mut self) -> EntryMut {
        self.index
            .entry_mut(self.data().parent_entry.to_owned())
            .unwrap()
    }

    /// Set the text content of the section.
    pub fn set_content(&mut self, content: &str) {
        // Get the lines of the old content.
        let old_content = self.content();
        let old_lines: Vec<_> = old_content.lines().collect();

        // Format and write the content.
        let content = process_text(&content);
        fs::write(format!("content/sections/{}.txt", self.id), &content);
        let new_lines: Vec<_> = content.lines().collect();
        self.data_mut().lines = new_lines.len();

        // Collect the line numbers that have threads.
        let thread_lines: HashSet<_> = self.data().comments.iter().map(|c| c.line).collect();

        // Map the old lines to the new lines.
        let mut line_map: HashMap<_, _> = thread_lines
            .into_iter()
            .map(|old_line_number| {
                let old_line = old_lines[old_line_number];
                // First, test that the line did not move.
                if new_lines.len() > old_line_number && old_line == new_lines[old_line_number] {
                    return (old_line_number, old_line_number);
                }
                // Or, search for the line somewhere else in the section.
                for (i, &new_line) in new_lines.iter().enumerate() {
                    if old_line == new_line {
                        return (old_line_number, i);
                    }
                }
                // Use Levenshtein distance to find closest line.
                let lines_and_distances: Vec<_> = new_lines
                    .iter()
                    .enumerate()
                    .map(|(i, &l)| (i, levenshtein(l, old_line)))
                    .collect();
                let closest_pair = lines_and_distances
                    .iter()
                    .min_by_key(|&(_, dist)| dist)
                    .unwrap();

                // If the closest line is more than half the length of the old line away,
                // use the old line.
                if closest_pair.1 > old_line.len() / 2 {
                    return (old_line_number, old_line_number);
                }

                (old_line_number, closest_pair.0)
            })
            .collect();

        // Transform the thread's lines.
        for comment in &mut self.data_mut().comments {
            comment.line = *line_map.get(&comment.line).unwrap();
        }
    }

    /// Change the location of the section.
    pub fn move_to(&mut self, position: Position<String, u32>) -> DataResult<()> {
        // Detach from current entry.
        let id = self.id;
        self.parent_entry_mut()
            .data_mut()
            .sections
            .retain(|&s| s != id);

        // Get new parent.
        let (mut new_parent_entry, new_index_in_parent) = position.resolve(&mut self.index)?;

        // Insert section.
        new_parent_entry
            .data_mut()
            .sections
            .insert(new_index_in_parent, self.id.clone());

        Ok(())
    }

    /// Remove the section from the journal.
    pub fn remove(mut self) {
        // Update parent entry.
        let id = self.id;
        self.parent_entry_mut()
            .data_mut()
            .sections
            .retain(|&s| s != id);

        // Update user reading history.
        let user_ids: Vec<_> = self.index.users.keys().cloned().collect();
        for user_id in user_ids {
            let mut user = self.index.user_mut(user_id).unwrap();
            user.data_mut().history.retain(|h| h.section != id);
        }

        // Update index registry.
        self.index.sections.remove(&self.id);

        // Archive files.
        let now = Utc::now().timestamp();
        fs::rename(
            format!("content/sections/{}.json", &self.id),
            format!("archived/section-{}-{now}", &self.id),
        );
        fs::rename(
            format!("content/sections/{}.txt", &self.id),
            format!("archived/content-{}-{now}", &self.id),
        );
        fs::remove_file(format!("content/sections/{}.index", &self.id));

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
        let heading = self.heading().cloned().unwrap_or_else(String::new);
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
        self.data_mut().length = search_index.total_word_count();
        let section = File::create(format!("content/sections/{id}.json")).unwrap();
        serde_json::to_writer_pretty(section, self.data()).unwrap();
    }
}
