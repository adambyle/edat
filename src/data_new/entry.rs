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

pub struct Entry<'index> {
    pub(super) index: &'index Index,
    pub(super) id: String,
}

macro_rules! immut_fns {
    () => {
        pub(super) fn data(&self) -> &EntryData {
            self.index.entries.get(&self.id).unwrap()
        }

        pub fn id(&self) -> &str {
            &self.id
        }

        pub fn title(&self) -> &str {
            &self.data().title
        }

        pub fn description(&self) -> &str {
            &self.data().description
        }

        pub fn summary(&self) -> &str {
            &self.data().summary
        }

        pub fn author_id(&self) -> &str {
            &self.data().author
        }

        pub fn author(&self) -> User {
            self.index.user(self.author_id().to_owned()).unwrap()
        }

        pub fn parent_volume_id(&self) -> &str {
            &self.data().parent_volume.0
        }

        pub fn parent_volume_part(&self) -> usize {
            self.data().parent_volume.1
        }

        pub fn parent_volume(&self) -> Volume {
            self.index
                .volume(self.data().parent_volume.0.to_owned())
                .unwrap()
        }

        pub fn index_in_parent(&self) -> usize {
            self.parent_volume()
                .entry_ids()
                .iter()
                .position(|e| e == &self.id)
                .unwrap()
        }

        pub fn section_count(&self) -> usize {
            self.data().sections.len()
        }

        pub fn section_ids(&self) -> &[u32] {
            &self.data().sections
        }

        pub fn sections(&self) -> impl Iterator<Item = Section> {
            self.section_ids()
                .iter()
                .map(|&s| self.index.section(s).unwrap())
        }

        pub fn search_index(&self) -> &search::Index {
            &self.data().search_index
        }
    };
}

impl Entry<'_> {
    immut_fns!();
}

pub struct EntryMut<'index> {
    pub(super) index: &'index mut Index,
    pub(super) id: String,
    pub(super) exists: bool,
}

impl EntryMut<'_> {
    pub(super) fn data_mut(&mut self) -> &mut EntryData {
        self.index.entries.get_mut(&self.id).unwrap()
    }

    immut_fns!();

    pub fn set_title(&mut self, title: &str) -> Result<(), DuplicateIdError<String>> {
        let title = process_text(title);

        // Changing the title also changes the id.
        let new_id = create_id(&title);

        if new_id != self.id {
            // Make sure the id is not a duplicate.
            if self.index.entries.contains_key(&new_id) {
                return Err(DuplicateIdError(new_id));
            }

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

            // Update index registry.
            let data = self.index.entries.remove(&self.id).unwrap();
            self.index.entries.insert(new_id.clone(), data);

            // Rename associated files.
            let _ = fs::rename(
                format!("content/entry/{}.json", &self.id),
                format!("content/entry/{}.json", &new_id),
            );
            let _ = fs::rename(
                format!("content/entry/{}.index", &self.id),
                format!("content/entry/{}.index", &new_id),
            );

            self.id = new_id;
        }

        self.data_mut().title = title;
        Ok(())
    }

    pub fn set_description(&mut self, description: &str) {
        self.data_mut().description = process_text(description);
    }

    pub fn set_summary(&mut self, summary: &str) {
        self.data_mut().summary = process_text(summary);
    }

    pub fn parent_volume_mut(&mut self) -> VolumeMut {
        self.index
            .volume_mut(self.data().parent_volume.0.to_owned())
            .unwrap()
    }

    pub fn move_to(&mut self, position: Position<(Volume, usize), Entry>) {
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
        let (mut new_parent_volume, new_parent_volume_part, new_index_in_parent) = match position {
            Position::StartOf((volume, part)) => {
                let volume = self.index.volume_mut(volume.id).unwrap();
                (volume, part, 0)
            }
            Position::EndOf((volume, part)) => {
                let index = volume.entry_count();
                let volume = self.index.volume_mut(volume.id).unwrap();
                (volume, part, index)
            }
            Position::Before(sibling) => {
                let index = sibling.index_in_parent();
                let volume = self
                    .index
                    .volume_mut(sibling.parent_volume_id().to_owned())
                    .unwrap();
                (volume, sibling.parent_volume_part(), index)
            }
            Position::After(sibling) => {
                let index = sibling.index_in_parent();
                let volume = self
                    .index
                    .volume_mut(sibling.parent_volume_id().to_owned())
                    .unwrap();
                (volume, sibling.parent_volume_part(), 1 + index)
            }
        };

        // Update new parent volume count.
        let current_count = new_parent_volume.parts_count();
        new_parent_volume.data_mut().volume_count = current_count.max(new_parent_volume_part);

        // Insert entry.
        new_parent_volume
            .data_mut()
            .entries
            .insert(new_index_in_parent, self.id.clone());
    }

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
        fs::rename(
            format!("content/entry/{}.json", &self.id),
            format!("/archive/entry-{}-{now}", &self.id),
        );

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
