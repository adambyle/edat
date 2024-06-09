use std::fs::{self, File};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::search;

use super::*;

#[derive(Serialize, Deserialize)]
pub struct VolumeData {
    pub(super) title: String,
    pub(super) old_ids: Vec<String>,
    pub(super) subtitle: Option<String>,
    pub(super) owner: String,
    pub(super) content_type: Kind,
    pub(super) volume_count: usize,
    pub(super) entries: Vec<String>,

    #[serde(skip)]
    pub(super) search_index: search::Index,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Kind {
    Journal,
    Archive,
    Diary,
    Cartoons,
    Creative,
    Featured,
}

pub struct Volume<'index> {
    pub(super) index: &'index Index,
    pub(super) id: String,
}

macro_rules! immut_fns {
    () => {
        pub(super) fn data(&self) -> &VolumeData {
            self.index.volumes.get(&self.id).unwrap()
        }

        pub fn id(&self) -> &str {
            &self.id
        }

        pub fn title(&self) -> &str {
            &self.data().title
        }

        pub fn subtitle(&self) -> Option<&String> {
            self.data().subtitle.as_ref()
        }

        pub fn owner_id(&self) -> &str {
            &self.data().owner
        }

        pub fn owner(&self) -> User {
            self.index.user(self.owner_id().to_owned()).unwrap()
        }

        pub fn kind(&self) -> Kind {
            self.data().content_type.clone()
        }

        pub fn index_in_list(&self) -> usize {
            self.index
                .volumes
                .keys()
                .position(|v| v == &self.id)
                .unwrap()
        }

        pub fn parts_count(&self) -> usize {
            self.data().volume_count
        }

        pub fn entry_count(&self) -> usize {
            self.data().entries.len()
        }

        pub fn entry_ids(&self) -> &[String] {
            &self.data().entries
        }

        pub fn entries(&self) -> impl Iterator<Item = Entry> {
            self.entry_ids()
                .iter()
                .map(|e| self.index.entry(e.clone()).unwrap())
        }

        pub fn intro(&self) -> String {
            fs::read_to_string(format!("content/volumes/{}.intro", self.id)).unwrap()
        }

        pub fn search_index(&self) -> &search::Index {
            &self.data().search_index
        }
    };
}

impl Volume<'_> {
    immut_fns!();
}

pub struct VolumeMut<'index> {
    pub(super) index: &'index mut Index,
    pub(super) id: String,
    pub(super) exists: bool,
}

impl VolumeMut<'_> {
    pub(super) fn data_mut(&mut self) -> &mut VolumeData {
        self.index.volumes.get_mut(&self.id).unwrap()
    }

    immut_fns!();

    pub fn set_title(&mut self, title: &str) -> Result<(), DuplicateIdError<String>> {
        let title = process_text(title);

        // Changing the title also changes the id.
        let new_id = create_id(&title);

        if new_id != self.id {
            // Make sure the id is not a duplicate.
            if self.index.volumes.contains_key(&new_id) {
                return Err(DuplicateIdError(new_id));
            }

            // Update index registry.
            let volume_index = self.index_in_list();
            let data = self
                .index
                .volumes
                .shift_remove_index(volume_index)
                .unwrap()
                .1;
            self.index
                .volumes
                .shift_insert(volume_index, new_id.clone(), data);
            self.index.save();

            // Update child entries.
            let entries = self.entry_ids().to_owned();
            for entry in entries {
                self.index
                    .entry_mut(entry)
                    .unwrap()
                    .data_mut()
                    .parent_volume
                    .0 = new_id.clone();
            }

            // Rename associated files.
            let _ = fs::rename(
                format!("content/volume/{}.json", &self.id),
                format!("content/volume/{}.json", &new_id),
            );
            let _ = fs::rename(
                format!("content/volume/{}.index", &self.id),
                format!("content/volume/{}.index", &new_id),
            );
            let _ = fs::rename(
                format!("content/volume/{}.intro", &self.id),
                format!("content/volume/{}.intro", &new_id),
            );

            self.id = new_id;
        }

        self.data_mut().title = title;
        Ok(())
    }

    pub fn set_subtitle(&mut self, subtitle: Option<String>) {
        self.data_mut().subtitle = subtitle;
    }

    pub fn set_kind(&mut self, kind: Kind) {
        self.data_mut().content_type = kind;
    }

    pub fn set_intro(&mut self, intro: String) {
        // Format and write the content.
        let content = process_text(&intro);
        fs::write(format!("content/volumes/{}.intro", self.id), content);
    }

    pub fn move_to(&mut self, position: Position<(), Volume>) {
        let id = self.id.clone();
        let volume = self.index.volumes.shift_remove(&id).unwrap();

        // Get new parent.
        let index = match position {
            Position::StartOf(()) => 0,
            Position::EndOf(()) => self.index.volumes.len(),
            Position::Before(sibling) => self
                .index
                .volumes
                .keys()
                .position(|v| v == &sibling.id)
                .unwrap(),
            Position::After(sibling) => {
                1 + self
                    .index
                    .volumes
                    .keys()
                    .position(|v| v == &sibling.id)
                    .unwrap()
            }
        };

        // Insert volume.
        self.index.volumes.shift_insert(index, id, volume);
    }

    pub fn remove(mut self) {
        // Orphan this volume's entries.
        let entry_ids = self.entry_ids().to_owned();
        for id in entry_ids {
            self.index.entry_mut(id).unwrap().remove();
        }

        // Update index registry.
        self.index.volumes.shift_remove(&self.id);

        // Archive files.
        let now = Utc::now().timestamp();
        let _ = fs::rename(
            format!("content/volume/{}.json", &self.id),
            format!("/archive/volume-{}-{now}", &self.id),
        );
        let _ = fs::rename(
            format!("content/volume/{}.intro", &self.id),
            format!("/archive/intro-{}-{now}", &self.id),
        );

        self.exists = false;
    }
}

impl Drop for VolumeMut<'_> {
    fn drop(&mut self) {
        if !self.exists {
            return;
        }

        // Create search index.
        let id = self.id.clone();
        let title = self.title().to_owned();
        let subtitle = self.subtitle().map(|s| s.clone()).unwrap_or(String::new());
        let intro = self.intro();
        let search_index = &mut self.data_mut().search_index;
        *search_index = search::Index::new();
        search_index.add_section("TITLE".to_owned(), &title);
        search_index.add_section("SUBTITLE".to_owned(), &subtitle);
        search_index.add_section("INTRO".to_owned(), &intro);
        fs::write(
            format!("content/volumes/{id}.index"),
            search_index.to_bytes(),
        )
        .unwrap();

        // Write data.
        let volume = File::create(format!("content/volumes/{id}.json")).unwrap();
        serde_json::to_writer_pretty(volume, self.data()).unwrap();
    }
}
