use std::{
    collections::HashMap,
    fs::{self, File},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Serialize, Deserialize)]
pub(super) struct UserData {
    pub(super) first_name: String,
    pub(super) last_name: String,
    pub(super) privilege: Privilege,
    pub(super) codes: Vec<String>,
    pub(super) widgets: Vec<String>,
    pub(super) history: Vec<HistoryEntry>,
    pub(super) preferences: HashMap<String, String>,
    pub(super) init: bool,
}

/// The privilege level of a user.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Privilege {
    /// The user can create content and use the terminal.
    Owner,

    /// The user can read content, make comments, and submit featured content.
    Member,
}

/// A wrapper around a user.
pub struct User<'index> {
    pub(super) index: &'index super::Index,
    pub(super) id: String,
}

macro_rules! immut_fns {
    () => {
        fn data(&self) -> &super::UserData {
            self.index.users.get(&self.id).unwrap()
        }

        pub(crate) fn id(&self) -> &str {
            &self.id
        }

        /// The user's first name.
        pub fn first_name(&self) -> &str {
            &self.data().first_name
        }

        /// The user's last name.
        pub fn last_name(&self) -> &str {
            &self.data().last_name
        }

        /// The user's full name.
        pub fn full_name(&self) -> String {
            format!("{} {}", self.first_name(), self.last_name())
        }

        /// The user's privilege level.
        pub fn privilege(&self) -> Privilege {
            self.data().privilege.clone()
        }

        /// The user's access codes.
        pub fn codes(&self) -> &[String] {
            &self.data().codes
        }

        /// Whether the user has the given access code.
        pub fn has_code(&self, code: &str) -> bool {
            self.data().codes.contains(&code.to_owned())
        }

        /// The user's homepage widgets.
        pub fn widgets(&self) -> &[String] {
            &self.data().widgets
        }

        /// The user's history, stored by section and ordered from latest to earliest.
        pub fn history(&self) -> Vec<(Section, SectionProgress)> {
            // Order the history entries from latest to earliest.
            let mut history = self.data().history.clone();
            history.sort_by_key(|h| -h.timestamp);

            // Map each history entry to a section and its progress.
            history
                .iter()
                .map(|h| {
                    let section = self.index.section(h.section).unwrap();
                    let progress = Self::internal_section_progress(h, &section);
                    (section, progress)
                })
                .collect()
        }

        /// The user's progress in a section.
        pub fn section_progress(&self, section: &Section) -> Option<SectionProgress> {
            self.data()
                .history
                .iter()
                .find(|h| h.section == section.id)
                .map(|h| Self::internal_section_progress(h, section))
        }

        fn internal_section_progress(history: &HistoryEntry, section: &Section) -> SectionProgress {
            use SectionProgress as S;
            match (history.ever_finished, history.line > 0) {
                (false, false) => unreachable!(),
                (false, true) => S::Reading {
                    // Section is being read but has never been finished.
                    last_read: history.timestamp,
                    line: history.line,
                },
                (true, false) => S::Finished {
                    // Section has been finished, and the user has not restarted it.
                    last_read: history.timestamp,
                },
                (true, true) => S::Rereading {
                    // Section has been finished, and the user has restarted it.
                    last_read: history.timestamp,
                    line: history.line,
                },
            }
        }

        /// The user's progress in an entry.
        pub fn entry_progress(&self, entry: &Entry) -> Option<EntryProgress> {
            // If none of the entry's sections have been started, then this entry is unstarted.
            if !entry
                .sections()
                .any(|s| self.section_progress(&s).is_some())
            {
                return None;
            }

            // Iterate through the entry's sections and look for one that is unstarted.
            // Or, find that the user is partway through a section.
            let mut last_read = 0;
            for section in entry.sections() {
                match self.section_progress(&section) {
                    None => {
                        return Some(EntryProgress::UpToSection {
                            section_id: section.id,
                            section_index: section.index_in_parent(),
                            out_of: entry.sections().count(),
                        })
                    }
                    Some(SectionProgress::Reading {
                        line,
                        last_read,
                    }) => {
                        return Some(EntryProgress::InSection {
                            section_id: section.id,
                            section_index: section.index_in_parent(),
                            out_of: entry.sections().count(),
                            line,
                            last_read,
                        })
                    }
                    Some(other) => last_read = last_read.max(other.timestamp()),
                }
            }

            // The user has finished the entry.
            Some(EntryProgress::Finished { last_read })
        }

        /// The user's preferences.
        pub fn preferences(&self) -> &HashMap<String, String> {
            &self.data().preferences
        }

        /// Whether the user is initialized.
        pub fn is_init(&self) -> bool {
            self.data().init
        }
    };
}

impl User<'_> {
    immut_fns!();
}

/// A mutable wrapper around a user.
pub struct UserMut<'index> {
    pub(super) index: &'index mut super::Index,
    pub(super) id: String,
}

impl UserMut<'_> {
    fn data_mut(&mut self) -> &mut super::UserData {
        self.index.users.get_mut(&self.id).unwrap()
    }

    pub fn as_immut(&self) -> User {
        User { index: &self.index, id: self.id.clone() }
    }

    immut_fns!();

    /// Set the user's name.
    ///
    /// This also changes the user's id, resulting in side effects in other resources.
    pub fn set_name(
        &mut self,
        first_name: String,
        last_name: String,
    ) -> DataResult<()> {
        let new_id = create_id(&format!("{}{}", first_name, last_name));

        if new_id != self.id {
            // Make sure the id is not a duplicate.
            if self.index.users.contains_key(&new_id) {
                return Err(DataError::DuplicateId(new_id));
            }

            // Update volume owners, entry authors, and section comments.
            for volume in self.index.volumes.values_mut() {
                if volume.owner == self.id {
                    volume.owner = new_id.clone();
                }
            }
            for entry in self.index.entries.values_mut() {
                if entry.author == self.id {
                    entry.author = new_id.clone();
                }
            }
            for section in self.index.sections.values_mut() {
                for comment in &mut section.comments {
                    if comment.author == self.id {
                        comment.author = new_id.clone();
                    }
                }
            }

            // Update index registry.
            let data = self.index.users.remove(&self.id).unwrap();
            self.index.users.insert(new_id.clone(), data);

            // Rename associated files.
            let _ = fs::rename(
                format!("users/{}.json", &self.id),
                format!("users/{}.json", &new_id),
            );

            self.id = new_id
        }

        self.data_mut().first_name = first_name;
        self.data_mut().last_name = last_name;
        Ok(())
    }

    /// Set the user's privilege level.
    pub fn set_privilege(&mut self, privilege: Privilege) {
        self.data_mut().privilege = privilege;
    }

    /// Add an access code for the user.
    pub fn add_code(&mut self, code: String) {
        self.data_mut().codes.push(code);
    }

    /// Remove an access code for the user.
    pub fn remove_code(&mut self, code: &str) {
        self.data_mut().codes.retain(|c| c != code);
    }

    /// Set the user's homepage widgets.
    pub fn set_widgets(&mut self, widgets: Vec<String>) {
        self.data_mut().widgets = widgets;
    }

    /// Update the progress of the user in a section.
    ///
    /// Return [`false`] if the section doesn't exist.
    pub fn reading_section(&mut self, section: u32, progress: usize) -> bool {
        // Update the history entry for this section's id if it exists.
        if let Some(history) = self
            .data_mut()
            .history
            .iter_mut()
            .find(|h| h.section == section)
        {
            history.timestamp = Utc::now().timestamp();
            history.line = progress;
            return true;
        }

        // Otherwise, add a new history entry.
        if self.index.sections.contains_key(&section) {
            self.data_mut().history.push(HistoryEntry {
                section: section,
                timestamp: Utc::now().timestamp(),
                ever_finished: false,
                line: progress,
            });
            return true;
        }

        // The section does not exist.
        false
    }

    /// Mark a section as finished for the user.
    ///
    /// Return [`false`] if the section doesn't exist.
    pub fn finished_section(&mut self, section: u32) -> bool {
        // Update the history entry for this section's id if it exists.
        if let Some(history) = self
            .data_mut()
            .history
            .iter_mut()
            .find(|h| h.section == section)
        {
            history.timestamp = Utc::now().timestamp();
            history.line = 0;
            history.ever_finished = true;
            return true;
        }

        // Otherwise, add a new history entry.
        if self.index.sections.contains_key(&section) {
            self.data_mut().history.push(HistoryEntry {
                section: section,
                timestamp: Utc::now().timestamp(),
                ever_finished: true,
                line: 0,
            });
            return true;
        }

        // The section does not exist.
        false
    }

    /// Mark an entry as finished for the user.
    ///
    /// Returns [`false`] if the entry doesn't exist.
    pub fn finished_entry(&mut self, entry: String) -> bool {
        let Ok(entry) = self.index.entry(entry) else {
            return false;
        };

        // Mark all child sections as finished.
        let ids = entry.section_ids().to_owned();
        for section in ids {
            self.finished_section(section);
        }

        true
    }

    /// Set a preference for the user.
    pub fn set_preference(&mut self, key: String, value: String) {
        self.data_mut().preferences.insert(key, value);
    }

    /// Remove a preference for the user.
    pub fn remove_preference(&mut self, key: &str) {
        self.data_mut().preferences.remove(key);
    }

    /// Mark the user as initialized.
    ///
    /// This is called when the user has chosen their homepage widgets
    /// and their read entries.
    pub fn init(&mut self) {
        self.data_mut().init = true;
    }
}

impl Drop for UserMut<'_> {
    fn drop(&mut self) {
        // Write data.
        let user = File::create(format!("users/{}.json", self.id)).unwrap();
        serde_json::to_writer_pretty(user, self.data()).unwrap();
    }
}
