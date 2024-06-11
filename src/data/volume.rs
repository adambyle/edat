use std::fs::{self, File};

use chrono::Utc;
use indexmap::IndexMap;
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

/// The kind of content this volume contains.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum Kind {
    /// The volume contains normal journal content.
    Journal,

    /// The volume contains pre-EDAT content.
    Archive,

    /// The volume contains diary content.
    Diary,

    /// The volume contains cartoon content.
    Cartoons,

    /// The volume contains creative-writing.
    Creative,

    /// The volume contains guest-user content.
    Featured,
}

/// A wrapper around a volume.
pub struct Volume<'index> {
    pub(super) index: &'index Index,
    pub(super) id: String,
}

macro_rules! immut_fns {
    () => {
        pub(super) fn data(&self) -> &VolumeData {
            self.index.volumes.get(&self.id).unwrap()
        }

        /// The volume id.
        pub fn id(&self) -> &str {
            &self.id
        }

        /// The volume title.
        pub fn title(&self) -> &str {
            &self.data().title
        }

        /// The volume subtitle, if any.
        pub fn subtitle(&self) -> Option<&String> {
            self.data().subtitle.as_ref()
        }

        /// The id of the owner.
        pub fn owner_id(&self) -> &str {
            &self.data().owner
        }

        /// Get a wrapper around the owner.
        pub fn owner(&self) -> User {
            self.index.user(self.owner_id().to_owned()).unwrap()
        }

        /// The kind of content in the volume.
        pub fn kind(&self) -> Kind {
            self.data().content_type.clone()
        }

        /// The index of the volume in the volume list.
        pub fn index_in_list(&self) -> usize {
            self.index
                .volumes
                .keys()
                .position(|v| v == &self.id)
                .unwrap()
        }

        /// The number of parts in the volume.
        pub fn parts_count(&self) -> usize {
            self.data().volume_count
        }

        /// The number of entries in the volume.
        pub fn entry_count(&self) -> usize {
            self.data().entries.len()
        }

        /// The ids of the entries in the volume.
        pub fn entry_ids(&self) -> &[String] {
            &self.data().entries
        }

        /// Get wrappers around the entries in the volume.
        pub fn entries(&self) -> impl Iterator<Item = Entry> {
            self.entry_ids()
                .iter()
                .map(|e| self.index.entry(e.clone()).unwrap())
        }

        /// Get wrappers around the entries in the volume, sorted by part.
        pub fn entries_by_part(&self) -> IndexMap<usize, Vec<Entry>> {
            let mut map = IndexMap::new();

            for entry in self.entries() {
                map.entry(entry.parent_volume_part())
                    .or_insert(Vec::new())
                    .push(entry);
            }

            map.sort_unstable_keys();

            map
        }

        /// Get the text content of the volume intro.
        pub fn intro(&self) -> String {
            fs::read_to_string(format!("content/volumes/{}.intro", self.id)).unwrap()
        }

        /// Get the search index.
        pub fn search_index(&self) -> &search::Index {
            &self.data().search_index
        }
    };
}

impl Volume<'_> {
    immut_fns!();
}

/// A mutable wrapper around a volume.
pub struct VolumeMut<'index> {
    pub(super) index: &'index mut Index,
    pub(super) id: String,
    pub(super) exists: bool,
}

impl VolumeMut<'_> {
    pub(super) fn data_mut(&mut self) -> &mut VolumeData {
        self.index.volumes.get_mut(&self.id).unwrap()
    }

    pub fn as_immut(&self) -> Volume {
        Volume {
            index: &self.index,
            id: self.id.clone(),
        }
    }

    immut_fns!();

    /// Set the volume title.
    pub fn set_title(&mut self, title: &str) -> DataResult<()> {
        let title = process_text(title);

        // Changing the title also changes the id.
        let new_id = create_id(&title);

        if new_id != self.id {
            // Make sure the id is not a duplicate.
            if self.index.volumes.contains_key(&new_id) {
                return Err(DataError::DuplicateId(new_id));
            }

            // Keep track of old ids.
            let old_id = self.id.clone();
            self.data_mut().old_ids.push(old_id);

            // Rename associated files.
            let _ = fs::rename(
                format!("content/volumes/{}.json", &self.id),
                format!("content/volumes/{}.json", &new_id),
            );
            let _ = fs::rename(
                format!("content/volumes/{}.index", &self.id),
                format!("content/volumes/{}.index", &new_id),
            );
            let _ = fs::rename(
                format!("content/volumes/{}.intro", &self.id),
                format!("content/volumes/{}.intro", &new_id),
            );

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
            self.id = new_id;
            self.index.save();
        }

        self.data_mut().title = title;
        Ok(())

    }

    /// Set the volume subtitle.
    pub fn set_subtitle(&mut self, subtitle: Option<&str>) {
        self.data_mut().subtitle = subtitle.map(|s| process_text(s));
    }

    /// Set the volume content kind.
    pub fn set_kind(&mut self, kind: Kind) {
        self.data_mut().content_type = kind;
    }

    /// Set the volume intro text.
    pub fn set_intro(&mut self, intro: &str) {
        // Format and write the content.
        let content = process_text(&intro);
        fs::write(format!("content/volumes/{}.intro", self.id), content);
    }

    /// Change the location of the volume.
    pub fn move_to(&mut self, position: Position<(), String>) -> DataResult<()> {
        let id = self.id.clone();
        let volume = self.index.volumes.shift_remove(&id).unwrap();

        // Get position.
        let index = position.resolve(&self.index)?;

        // Insert volume.
        self.index.volumes.shift_insert(index, id, volume);

        Ok(())
    }

    /// Remove the volume from the journal.
    pub fn remove(mut self) {
        // Orphan this volume's entries.
        let entry_ids = self.entry_ids().to_owned();
        for id in entry_ids {
            self.index.entry_mut(id).unwrap().remove();
        }

        // Update index registry.
        self.index.volumes.shift_remove(&self.id);
        self.index.save();

        // Archive files.
        let now = Utc::now().timestamp();
        let _ = fs::rename(
            format!("content/volumes/{}.json", &self.id),
            format!("archive/volume-{}-{now}", &self.id),
        );
        let _ = fs::rename(
            format!("content/volumes/{}.intro", &self.id),
            format!("archive/intro-{}-{now}", &self.id),
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
