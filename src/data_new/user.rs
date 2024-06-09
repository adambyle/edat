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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Privilege {
    Owner,
    Reader,
}

pub struct User<'index> {
    pub(super) index: &'index super::Index,
    pub(super) id: String,
}

macro_rules! immut_fns {
    () => {
        fn data(&self) -> &super::UserData {
            self.index.users.get(&self.id).unwrap()
        }
    };
}

impl User<'_> {
    immut_fns!();

    pub fn first_name(&self) -> &str {
        &self.data().first_name
    }

    pub fn last_name(&self) -> &str {
        &self.data().last_name
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name(), self.last_name())
    }

    pub fn privilege(&self) -> Privilege {
        self.data().privilege.clone()
    }

    pub fn codes(&self) -> &[String] {
        &self.data().codes
    }

    pub fn widgets(&self) -> &[String] {
        &self.data().widgets
    }

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

    pub fn section_progress(&self, section: &Section) -> SectionProgress {
        self.data()
            .history
            .iter()
            .find(|h| h.section == section.id)
            .map(|h| Self::internal_section_progress(h, section))
            .unwrap_or(SectionProgress::Unstarted)
    }

    fn internal_section_progress(history: &HistoryEntry, section: &Section) -> SectionProgress {
        use SectionProgress as S;
        match (history.ever_finished, history.progress > 0) {
            (false, false) => unreachable!(),
            (false, true) => S::Reading {
                // Section is being read but has never been finished.
                last_read: history.timestamp,
                progress: LineProgress(history.progress, section.lines()),
            },
            (true, false) => S::Finished {
                // Section has been finished, and the user has not restarted it.
                last_read: history.timestamp,
            },
            (true, true) => S::Rereading {
                // Section has been finished, and the user has restarted it.
                last_read: history.timestamp,
                progress: LineProgress(history.progress, section.lines()),
            },
        }
    }

    pub fn entry_progress(&self, entry: &Entry) -> EntryProgress {
        // If none of the entry's sections have been started, then this entry is unstarted.
        if !entry
            .sections()
            .any(|s| self.section_progress(&s).started())
        {
            return EntryProgress::Unstarted;
        }

        // Iterate through the entry's sections and look for one that is unstarted.
        // Or, find that the user is partway through a section.
        for section in entry.sections() {
            match self.section_progress(&section) {
                SectionProgress::Unstarted => {
                    return EntryProgress::UpToSection {
                        section_id: section.id,
                        section_index: section.index_in_parent(),
                        out_of: entry.sections().count(),
                    }
                }
                SectionProgress::Reading {
                    progress,
                    last_read,
                } => {
                    return EntryProgress::InSection {
                        section_id: section.id,
                        section_index: section.index_in_parent(),
                        out_of: entry.sections().count(),
                        progress,
                        last_read,
                    }
                }
                _ => (),
            }
        }

        // The user has finished the entry.
        EntryProgress::Finished
    }

    pub fn preferences(&self) -> &HashMap<String, String> {
        &self.data().preferences
    }
}

pub struct UserMut<'index> {
    pub(super) index: &'index mut super::Index,
    pub(super) id: String,
}

impl UserMut<'_> {
    fn data_mut(&mut self) -> &mut super::UserData {
        self.index.users.get_mut(&self.id).unwrap()
    }

    immut_fns!();

    // TODO move into immut_fns.

    pub fn set_name(
        &mut self,
        first_name: String,
        last_name: String,
    ) -> Result<(), DuplicateIdError<String>> {
        let new_id = create_id(&format!("{} {}", first_name, last_name));

        if new_id != self.id {
            // Make sure the id is not a duplicate.
            if self.index.users.contains_key(&new_id) {
                return Err(DuplicateIdError(new_id));
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
                format!("/users/{}.json", &self.id),
                format!("/users/{}.json", &new_id),
            );

            self.id = new_id
        }

        self.data_mut().first_name = first_name;
        self.data_mut().last_name = last_name;
        Ok(())
    }

    pub fn set_privilege(&mut self, privilege: Privilege) {
        self.data_mut().privilege = privilege;
    }

    pub fn add_code(&mut self, code: String) {
        self.data_mut().codes.push(code);
    }

    pub fn remove_code(&mut self, code: &str) {
        self.data_mut().codes.retain(|c| c != code);
    }

    pub fn set_widgets(&mut self, widgets: Vec<String>) {
        self.data_mut().widgets = widgets;
    }

    pub fn reading_section(&mut self, section: Section, progress: usize) {
        // Update the history entry for this section's id if it exists.
        if let Some(history) = self
            .data_mut()
            .history
            .iter_mut()
            .find(|h| h.section == section.id)
        {
            history.timestamp = Utc::now().timestamp();
            history.progress = progress;
        }
        // Otherwise, add a new history entry.
        else {
            self.data_mut().history.push(HistoryEntry {
                section: section.id,
                timestamp: Utc::now().timestamp(),
                ever_finished: false,
                progress,
            });
        }
    }

    pub fn finished_section(&mut self, section: Section) {
        // Update the history entry for this section's id if it exists.
        if let Some(history) = self
            .data_mut()
            .history
            .iter_mut()
            .find(|h| h.section == section.id)
        {
            history.timestamp = Utc::now().timestamp();
            history.progress = 0;
            history.ever_finished = true;
        }
        // Otherwise, add a new history entry.
        else {
            self.data_mut().history.push(HistoryEntry {
                section: section.id,
                timestamp: Utc::now().timestamp(),
                ever_finished: true,
                progress: 0,
            });
        }
    }

    pub fn finished_entry(&mut self, entry: Entry) {
        // Mark all child sections as finished.
        for section in entry.sections() {
            self.finished_section(section);
        }
    }

    pub fn set_preference(&mut self, key: String, value: String) {
        self.data_mut().preferences.insert(key, value);
    }

    pub fn remove_preference(&mut self, key: &str) {
        self.data_mut().preferences.remove(key);
    }
}

impl Drop for UserMut<'_> {
    fn drop(&mut self) {
        // Write data.
        let user = File::create(format!("users/{}.json", self.id)).unwrap();
        serde_json::to_writer_pretty(user, self.data()).unwrap();
    }
}
