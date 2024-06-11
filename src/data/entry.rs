use std::fs::{self, File};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::search;

use super::*;

#[derive(Serialize, Deserialize)]
pub(super) struct EntryData {
    pub(super) title: String,
    pub(super) old_ids: Vec<String>,
    pub(super) description: String,
    pub(super) summary: String,
    pub(super) author: String,
    pub(super) parent_volume: (String, usize),
    pub(super) sections: Vec<u32>,

    #[serde(skip)]
    pub(super) search_index: search::Index,
}

/// A wrapper around an entry.
pub struct Entry<'index> {
    pub(super) index: &'index Index,
    pub(super) id: String,
}

macro_rules! immut_fns {
    () => {
        pub(super) fn data(&self) -> &EntryData {
            self.index.entries.get(&self.id).unwrap()
        }

        /// The entry id.
        pub fn id(&self) -> &str {
            &self.id
        }

        /// The entry title.
        pub fn title(&self) -> &str {
            &self.data().title
        }

        /// The entry description (brief explanation).
        pub fn description(&self) -> &str {
            &self.data().description
        }

        /// The entry summary (longer explanation).
        pub fn summary(&self) -> &str {
            &self.data().summary
        }

        /// The id of the author.
        pub fn author_id(&self) -> &str {
            &self.data().author
        }

        /// Get a wrapper around the author.
        pub fn author(&self) -> User {
            self.index.user(self.author_id().to_owned()).unwrap()
        }

        /// The id of the parent volume.
        pub fn parent_volume_id(&self) -> &str {
            &self.data().parent_volume.0
        }

        /// The part or subvolume this entry is in.
        pub fn parent_volume_part(&self) -> usize {
            self.data().parent_volume.1
        }

        pub(super) fn index_in_parent(&self) -> usize {
            self.parent_volume()
                .entry_ids()
                .iter()
                .position(|e| e == &self.id)
                .unwrap()
        }

        /// The index of this entry in its parent volume.
        pub fn index_in_parent_volume_part(&self) -> usize {
            self.parent_volume()
                .entries()
                .filter(|e| e.parent_volume_part() == self.parent_volume_part())
                .position(|e| e.id == self.id)
                .unwrap()
        }

        /// The number of sections in this entry.
        pub fn section_count(&self) -> usize {
            self.data().sections.len()
        }

        /// The ids of the sections in this entry.
        pub fn section_ids(&self) -> &[u32] {
            &self.data().sections
        }

        /// Get the search index for this entry.
        pub fn search_index(&self) -> &search::Index {
            &self.data().search_index
        }
    };
}

impl Entry<'_> {
    immut_fns!();
}

impl<'index> Entry<'index> {
    /// The parent index.
    pub fn index(&self) -> &'index Index {
        &self.index
    }

    /// Get a wrapper around the parent volume.
    pub fn parent_volume(&self) -> Volume<'index> {
        self.index
            .volume(self.data().parent_volume.0.to_owned())
            .unwrap()
    }

    /// Get wrappers around the sections in this entry.
    pub fn sections(&self) -> impl Iterator<Item = Section<'index>> {
        self.section_ids()
            .to_owned()
            .into_iter()
            .map(|s| self.index.section(s).unwrap())
    }
}

/// A mutable wrapper around an entry.
pub struct EntryMut<'index> {
    pub(super) index: &'index mut Index,
    pub(super) id: String,
    pub(super) exists: bool,
}

impl EntryMut<'_> {
    pub(super) fn data_mut(&mut self) -> &mut EntryData {
        self.index.entries.get_mut(&self.id).unwrap()
    }

    pub fn as_immut(&self) -> Entry {
        Entry {
            index: &self.index,
            id: self.id.clone(),
        }
    }

    /// Get a wrapper around the parent volume.
    pub fn parent_volume(&self) -> Volume {
        self.index
            .volume(self.data().parent_volume.0.to_owned())
            .unwrap()
    }

    /// Get wrappers around the sections in this entry.
    pub fn sections(&self) -> impl Iterator<Item = Section> {
        self.section_ids()
            .iter()
            .map(|&s| self.index.section(s).unwrap())
    }

    immut_fns!();

    /// Set the entry title.
    ///
    /// This also changes the entry's id, resulting in side effects in other resources.
    pub fn set_title(&mut self, title: &str) -> DataResult<()> {
        let title = process_text(title);

        // Changing the title also changes the id.
        let new_id = create_id(&title);

        if new_id != self.id {
            // Make sure the id is not a duplicate.
            if self.index.entries.contains_key(&new_id) {
                return Err(DataError::DuplicateId(new_id));
            }

            // Keep track of old ids.
            let old_id = self.id.clone();
            self.data_mut().old_ids.push(old_id);

            // Update parent volume.
            let position = self.index_in_parent();
            self.parent_volume_mut().data_mut().entries[position] = new_id.clone();

            // Update child sections.
            let sections = self.section_ids().to_owned();
            for section in sections {
                self.index
                    .section_mut(section)
                    .unwrap()
                    .data_mut()
                    .parent_entry = new_id.clone();
            }

            // Rename associated files.
            let _ = fs::rename(
                format!("content/entries/{}.json", &self.id),
                format!("content/entries/{}.json", &new_id),
            );
            let _ = fs::rename(
                format!("content/entries/{}.index", &self.id),
                format!("content/entries/{}.index", &new_id),
            );

            // Update index registry.
            let data = self.index.entries.remove(&self.id).unwrap();
            self.index.entries.insert(new_id.clone(), data);
            self.id = new_id;
        }

        self.data_mut().title = title;
        Ok(())
    }

    /// Set the entry description (brief explanation).
    pub fn set_description(&mut self, description: &str) {
        self.data_mut().description = process_text(description);
    }

    /// Set the entry summary (longer explanation).
    pub fn set_summary(&mut self, summary: &str) {
        self.data_mut().summary = process_text(summary);
    }

    /// Get the parent volume for mutation.
    pub fn parent_volume_mut(&mut self) -> VolumeMut {
        self.index
            .volume_mut(self.data().parent_volume.0.to_owned())
            .unwrap()
    }

    /// Change the location of the entry.
    pub fn move_to(&mut self, position: Position<(String, usize), String>) -> DataResult<()> {
        // Detach from current volume.
        let parent_volume_id = self.parent_volume_id().to_owned();
        let parent_volume = self.index.volumes.get_mut(&parent_volume_id).unwrap();
        parent_volume.entries.retain(|e| e != &self.id);

        // Update old parent volume count.
        parent_volume.volume_count = parent_volume
            .entries
            .iter()
            .map(|e| self.index.entries.get(e).unwrap().parent_volume.1)
            .max()
            .unwrap_or(0);

        // Get new parent.
        let (mut new_parent_volume, new_parent_volume_part, new_index_in_parent) =
            position.resolve(&mut self.index)?;

        // Update new parent volume count.
        let current_count = new_parent_volume.parts_count();
        new_parent_volume.data_mut().volume_count = current_count.max(new_parent_volume_part);

        // Insert entry.
        new_parent_volume
            .data_mut()
            .entries
            .insert(new_index_in_parent, self.id.clone());

        Ok(())
    }

    /// Remove the entry from the journal.
    pub fn remove(mut self) {
        // Orphan this entry's sections.
        let section_ids = self.section_ids().to_owned();
        for id in section_ids {
            self.index.section_mut(id).unwrap().remove();
        }

        // Update parent volume.
        let parent_volume_id = self.parent_volume_id().to_owned();
        let parent_volume = self.index.volumes.get_mut(&parent_volume_id).unwrap();
        parent_volume.entries.retain(|e| e != &self.id);

        // Update parent volume count.
        parent_volume.volume_count = parent_volume
            .entries
            .iter()
            .map(|e| self.index.entries.get(e).unwrap().parent_volume.1)
            .max()
            .unwrap_or(0);

        // Update index registry.
        self.index.entries.remove(&self.id);

        // Archive files.
        let now = Utc::now().timestamp();
        let _ = fs::rename(
            format!("content/entries/{}.json", &self.id),
            format!("archived/entry-{}-{now}", &self.id),
        );
        fs::remove_file(format!("content/entries/{}.index", &self.id));

        // Prevent saving on drop.
        self.exists = false;
    }
}

impl Drop for EntryMut<'_> {
    fn drop(&mut self) {
        if !self.exists {
            return;
        }

        // Create search index.
        let id = self.id.clone();
        let title = self.title().to_owned();
        let description = self.description().to_owned();
        let summary = self.summary().to_owned();
        let search_index = &mut self.data_mut().search_index;
        *search_index = search::Index::new();
        search_index.add_section("TITLE".to_owned(), &title);
        search_index.add_section("DESCRIPTION".to_owned(), &description);
        search_index.add_section("SUMMARY".to_owned(), &summary);
        fs::write(
            format!("content/entries/{id}.index"),
            search_index.to_bytes(),
        )
        .unwrap();

        // Write data.
        let entry = File::create(format!("content/entries/{id}.json")).unwrap();
        serde_json::to_writer_pretty(entry, self.data()).unwrap();
    }
}
