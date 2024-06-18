use std::{
    collections::HashMap,
    fs::{self, File},
};

use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::*;

use crate::search;

#[derive(Serialize, Deserialize)]
struct IndexFile {
    volumes: Vec<String>,
    next_section_id: u32,
}

/// Wrapper around all journal data.
pub struct Index {
    pub(super) users: HashMap<String, UserData>,
    pub(super) volumes: IndexMap<String, VolumeData>,
    pub(super) entries: HashMap<String, EntryData>,
    pub(super) sections: HashMap<u32, SectionData>,
    pub(super) next_section_id: u32,
}

impl Index {
    /// Force a save of every resource in the index.
    /// 
    /// This is useful for regenerating search indices when the indexing
    /// algorithm changes.
    pub fn save_all(&mut self) {
        let volume_ids: Vec<_> = self.volumes.keys().cloned().collect();
        for id in volume_ids {
            drop(self.volume_mut(id));
        }

        let entry_ids: Vec<_> = self.entries.keys().cloned().collect();
        for id in entry_ids {
            drop(self.entry_mut(id));
        }

        let section_ids: Vec<_> = self.sections.keys().cloned().collect();
        for id in section_ids {
            drop(self.section_mut(id));
        }

        let user_ids: Vec<_> = self.users.keys().cloned().collect();
        for id in user_ids {
            drop(self.user_mut(id));
        }
    }
    
    /// Read data from the filesystem and construct and interface to the journal data.
    pub fn init() -> Self {
        let index_file =
            fs::read_to_string("content/index.json").expect("error reading index file");
        let index_file: IndexFile =
            serde_json::from_str(&index_file).expect("index file invalid json");

        let users = fs::read_to_string("users/users.json").expect("error reading users file");
        let users: Vec<String> = serde_json::from_str(&users).expect("users file invalid json");
        let users = users
            .into_iter()
            .map(|u| {
                let user = fs::read_to_string(format!("users/{u}.json"))
                    .expect(&format!("error reading user {u} file"));
                let user = serde_json::from_str(&user).expect(&format!("user {u} invalid json"));
                (u, user)
            })
            .collect();

        let volumes: IndexMap<String, VolumeData> = index_file
            .volumes
            .into_iter()
            .map(|v| {
                let volume = fs::read_to_string(format!("content/volumes/{v}.json"))
                    .expect(&format!("error reading volume {v} file"));
                let mut volume: VolumeData =
                    serde_json::from_str(&volume).expect(&format!("volume {v} invalid json"));
                volume.search_index = search::Index::from_bytes(
                    &fs::read(format!("content/volumes/{v}.index"))
                        .expect(&format!("error reading volume {v} index file")),
                );
                (v, volume)
            })
            .collect();

        let entries: HashMap<String, EntryData> = volumes
            .values()
            .map(|v| v.entries.iter())
            .flatten()
            .map(|e| {
                let entry = fs::read_to_string(format!("content/entries/{e}.json"))
                    .expect(&format!("error reading entry {e} file"));
                let mut entry: EntryData =
                    serde_json::from_str(&entry).expect(&format!("entry {e} invalid json"));
                entry.search_index = search::Index::from_bytes(
                    &fs::read(format!("content/entries/{e}.index"))
                        .expect(&format!("error reading entry {e} index file")),
                );
                (e.clone(), entry)
            })
            .collect();

        let sections = entries
            .values()
            .map(|e| e.sections.iter())
            .flatten()
            .map(|&s| {
                let section = fs::read_to_string(format!("content/sections/{s}.json"))
                    .expect(&format!("error reading section {s} file"));
                let mut section: SectionData =
                    serde_json::from_str(&section).expect(&format!("section {s} invalid json"));
                section.search_index = search::Index::from_bytes(
                    &fs::read(format!("content/sections/{s}.index"))
                        .expect(&format!("error reading section {s} index file")),
                );
                (s, section)
            })
            .collect();

        Index {
            users,
            volumes,
            entries,
            sections,
            next_section_id: index_file.next_section_id,
        }
    }

    pub(super) fn save(&self) {
        let index_file = serde_json::to_string_pretty(&IndexFile {
            volumes: self.volumes.keys().cloned().collect(),
            next_section_id: self.next_section_id,
        })
        .expect("error serializing index file");
        fs::write("content/index.json", index_file).expect("error writing index file");
    }

    /// Get the user with the specified id.
    pub fn user(&self, id: String) -> DataResult<User> {
        if self.users.contains_key(&id) {
            Ok(User { index: self, id })
        } else {
            Err(DataError::MissingResource("user", id))
        }
    }

    /// Get all the users.
    pub fn users(&self) -> impl Iterator<Item = User> {
        self.users.iter().map(|(id, u)| User {
            index: self,
            id: id.clone(),
        })
    }

    /// Get the user with the specified id for mutation.
    pub fn user_mut(&mut self, id: String) -> DataResult<UserMut> {
        if self.users.contains_key(&id) {
            Ok(UserMut { index: self, id })
        } else {
            Err(DataError::MissingResource("user", id))
        }
    }

    /// Add a user.
    pub fn create_user(&mut self, first_name: String, last_name: String) -> DataResult<UserMut> {
        // Make sure this id is not a duplicate.
        let id = format!("{}{}", first_name.to_lowercase(), last_name.to_lowercase());
        if self.users.contains_key(&id) {
            return Err(DataError::DuplicateId(id));
        }

        let user = UserData {
            first_name,
            last_name,
            privilege: user::Privilege::Member,
            codes: vec![],
            widgets: vec![],
            history: vec![],
            preferences: HashMap::new(),
            init: false,
        };

        self.users.insert(id.clone(), user);

        Ok(self.user_mut(id).unwrap())
    }

    /// Get the volume with the specified id.
    pub fn volume(&self, id: String) -> DataResult<Volume> {
        if self.volumes.contains_key(&id) {
            return Ok(Volume { index: self, id });
        }

        // See if any volumes have this as an old id.
        for (actual_id, volume) in &self.volumes {
            if volume.old_ids.contains(&id) {
                return Ok(Volume {
                    index: self,
                    id: actual_id.clone(),
                });
            }
        }

        Err(DataError::MissingResource("volume", id))
    }

    /// Get all volumes.
    pub fn volumes(&self) -> impl Iterator<Item = Volume> {
        self.volumes.iter().map(|(id, v)| Volume {
            index: self,
            id: id.clone(),
        })
    }

    /// Get the volume with the specified id for mutation.
    pub fn volume_mut(&mut self, id: String) -> DataResult<VolumeMut> {
        if self.volumes.contains_key(&id) {
            return Ok(VolumeMut {
                index: self,
                id,
                exists: true,
            });
        }

        // See if any volumes have this as an old id.
        for (actual_id, volume) in &self.volumes {
            if volume.old_ids.contains(&id) {
                let id = actual_id.clone();
                return Ok(VolumeMut {
                    index: self,
                    id,
                    exists: true,
                });
            }
        }

        Err(DataError::MissingResource("volume", id))
    }

    /// Add a volume.
    ///
    /// Returns the volume's id.
    pub fn create_volume(
        &mut self,
        title: &str,
        subtitle: Option<&str>,
        owner: String,
        position: Position<(), String>,
    ) -> DataResult<VolumeMut> {
        // Make sure this id is not a duplicate.
        let id = create_id(title);
        if self.volumes.contains_key(&id) {
            return Err(DataError::DuplicateId(id));
        }

        // Get position.
        let index = position.resolve(self)?;

        let volume = VolumeData {
            title: process_text(title),
            old_ids: Vec::new(),
            subtitle: subtitle.map(|s| process_text(s)),
            owner,
            content_type: volume::Kind::Journal,
            volume_count: 0,
            entries: Vec::new(),
            search_index: search::Index::new(),
        };

        // Create files.
        File::create(format!("content/volumes/{id}.json")).unwrap();
        File::create(format!("content/volumes/{id}.intro")).unwrap();
        File::create(format!("content/volumes/{id}.index")).unwrap();

        // Insert into index.
        self.volumes.shift_insert(index, id.clone(), volume);
        self.save();

        Ok(self.volume_mut(id).unwrap())
    }

    /// Get the entry with the specified id.
    pub fn entry(&self, id: String) -> DataResult<Entry> {
        if self.entries.contains_key(&id) {
            return Ok(Entry { index: self, id });
        }

        // See if any entries have this as an old id.
        for (actual_id, entry) in &self.entries {
            if entry.old_ids.contains(&id) {
                return Ok(Entry {
                    index: self,
                    id: actual_id.clone(),
                });
            }
        }

        Err(DataError::MissingResource("entry", id))
    }

    /// Get all entries.
    pub fn entries(&self) -> impl Iterator<Item = Entry> {
        self.entries.iter().map(|(id, e)| Entry {
            index: self,
            id: id.clone(),
        })
    }

    /// Get the entry with the specified id for mutation.
    pub fn entry_mut(&mut self, id: String) -> DataResult<EntryMut> {
        if self.entries.contains_key(&id) {
            return Ok(EntryMut {
                index: self,
                id,
                exists: true,
            });
        }

        // See if any entries have this as an old id.
        for (actual_id, entry) in &self.entries {
            if entry.old_ids.contains(&id) {
                let id = actual_id.clone();
                return Ok(EntryMut {
                    index: self,
                    id,
                    exists: true,
                });
            }
        }

        Err(DataError::MissingResource("entry", id))
    }

    /// Add an entry.
    ///
    /// Returns the entry's id.
    pub fn create_entry(
        &mut self,
        title: &str,
        description: &str,
        summary: &str,
        author: String,
        position: Position<(String, usize), String>,
    ) -> DataResult<EntryMut> {
        // Make sure this id is not a duplicate.
        let id = create_id(&title);
        if self.entries.contains_key(&id) {
            return Err(DataError::DuplicateId(id));
        }

        // Get position.
        let (mut parent_volume, parent_volume_part, index_in_parent) = position.resolve(self)?;

        let entry = EntryData {
            title: process_text(title),
            old_ids: Vec::new(),
            description: process_text(description),
            summary: process_text(summary),
            author,
            parent_volume: (parent_volume.id.to_owned(), parent_volume_part),
            sections: Vec::new(),
            search_index: search::Index::new(),
        };

        // Create files.
        File::create(format!("content/entries/{id}.json")).unwrap();
        File::create(format!("content/entries/{id}.index")).unwrap();

        // Insert into parent volume.
        parent_volume
            .data_mut()
            .entries
            .insert(index_in_parent, id.clone());
        let current_part_count = parent_volume.parts_count();
        parent_volume.data_mut().volume_count = current_part_count.max(1 + parent_volume_part);
        drop(parent_volume);

        // Insert into index.
        self.entries.insert(id.clone(), entry);

        Ok(self.entry_mut(id).unwrap())
    }

    /// Get the section with the specified id.
    pub fn section(&self, id: u32) -> DataResult<Section> {
        if self.sections.contains_key(&id) {
            return Ok(Section { index: self, id });
        } else {
            Err(DataError::MissingResource("section", id.to_string()))
        }
    }

    /// Get all sections.
    pub fn sections(&self) -> impl Iterator<Item = Section> {
        self.sections.iter().map(|(id, s)| Section {
            index: self,
            id: id.clone(),
        })
    }

    /// Get the section with the specified id for mutation.
    pub fn section_mut(&mut self, id: u32) -> DataResult<SectionMut> {
        if self.sections.contains_key(&id) {
            return Ok(SectionMut {
                index: self,
                id,
                exists: true,
            });
        } else {
            Err(DataError::MissingResource("section", id.to_string()))
        }
    }

    /// Add a section.
    ///
    /// Returns the section's id.
    pub fn create_section(
        &mut self,
        heading: Option<&str>,
        description: &str,
        summary: &str,
        date: NaiveDate,
        position: Position<String, u32>,
    ) -> DataResult<SectionMut> {
        let id = self.next_section_id;

        // Make sure this id is not a duplicate.
        if self.sections.contains_key(&id) {
            return Err(DataError::DuplicateId(id.to_string()));
        }

        // Get position.
        let (mut parent_entry, index_in_parent) = position.resolve(self)?;

        let section = SectionData {
            heading: heading.map(|h| process_text(h)),
            description: process_text(description),
            summary: process_text(summary),
            status: section::Status::Missing,
            date: date.format("%Y-%m-%d").to_string(),
            comments: Vec::new(),
            parent_entry: parent_entry.id.to_owned(),
            length: 0,
            lines: 0,
            perspectives: Vec::new(),
            search_index: search::Index::new(),
        };

        // Create files.
        File::create(format!("content/sections/{id}.txt")).unwrap();
        File::create(format!("content/sections/{id}.json")).unwrap();
        File::create(format!("content/sections/{id}.index")).unwrap();

        // Insert into parent entry.
        parent_entry.data_mut().sections.insert(index_in_parent, id);
        drop(parent_entry);

        // Insert into index.
        self.sections.insert(id, section);

        // Increase section id.
        self.next_section_id += 1;
        self.save();

        Ok(self.section_mut(id).unwrap())
    }

    /// Get the id for the next created section.
    pub fn next_section_id(&self) -> u32 {
        self.next_section_id
    }
}
