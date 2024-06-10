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

        let volumes: IndexMap<String, super::VolumeData> = index_file
            .volumes
            .into_iter()
            .map(|v| {
                let volume = fs::read_to_string(format!("content/volumes/{v}.json"))
                    .expect(&format!("error reading volume {v} file"));
                let mut volume: super::VolumeData =
                    serde_json::from_str(&volume).expect(&format!("volume {v} invalid json"));
                volume.search_index = search::Index::from_bytes(
                    &fs::read(format!("content/volumes/{v}.index"))
                        .expect(&format!("error reading volume {v} index file")),
                );
                (v, volume)
            })
            .collect();

        let entries: HashMap<String, super::EntryData> = volumes
            .values()
            .map(|v| v.entries.iter())
            .flatten()
            .map(|e| {
                let entry = fs::read_to_string(format!("content/entries/{e}.json"))
                    .expect(&format!("error reading entry {e} file"));
                let mut entry: super::EntryData =
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
                let section = fs::read_to_string(format!("content/section/{s}.json"))
                    .expect(&format!("error reading section {s} file"));
                let mut section: super::SectionData =
                    serde_json::from_str(&section).expect(&format!("section {s} invalid json"));
                section.search_index = search::Index::from_bytes(
                    &fs::read(format!("content/section/{s}.index"))
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
            volumes: self.volumes.keys().map(|k| k.clone()).collect(),
            next_section_id: self.next_section_id,
        })
        .expect("error serializing index file");
        fs::write("content/index.json", index_file).expect("error writing index file");
    }

    /// Get the user with the specified id.
    pub fn user(&self, id: String) -> Option<super::User> {
        self.users
            .contains_key(&id)
            .then_some(super::User { index: self, id })
    }

    /// Get the user with the specified id for mutation.
    pub fn user_mut(&mut self, id: String) -> Option<super::UserMut> {
        self.users
            .contains_key(&id)
            .then_some(super::UserMut { index: self, id })
    }

    /// Add a user.
    pub fn create_user(&mut self, first_name: String, last_name: String) {
        let id = format!("{}{}", first_name.to_lowercase(), last_name.to_lowercase());

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

        self.users.insert(id, user);
    }

    /// Get the volume with the specified id.
    pub fn volume(&self, id: String) -> Option<super::Volume> {
        if self.volumes.contains_key(&id) {
            Some(super::Volume { index: self, id })
        } else {
            // See if any volumes had the provided id as an old id.
            self.volumes
                .values()
                .any(|e| e.old_ids.contains(&id))
                .then(|| super::Volume { index: self, id })
        }
    }

    /// Get the volume with the specified id for mutation.
    pub fn volume_mut(&mut self, id: String) -> Option<super::VolumeMut> {
        if self.volumes.contains_key(&id) {
            Some(super::VolumeMut {
                index: self,
                id,
                exists: true,
            })
        } else {
            // See if any volumes had the provided id as an old id.
            self.volumes
                .values()
                .any(|e| e.old_ids.contains(&id))
                .then(|| super::VolumeMut {
                    index: self,
                    id,
                    exists: true,
                })
        }
    }

    /// Add a volume.
    ///
    /// Returns the volume's id.
    pub fn create_volume(
        &mut self,
        title: String,
        subtitle: Option<String>,
        owner: User,
        position: Position<(), Volume>,
    ) -> Result<String, DuplicateIdError<String>> {
        // Make sure this id is not a duplicate.
        let id = create_id(&title);
        if self.volumes.contains_key(&id) {
            return Err(DuplicateIdError(id));
        }

        // Get position.
        let index = position.resolve(self);

        let volume = VolumeData {
            title: process_text(&title),
            old_ids: Vec::new(),
            subtitle: subtitle.map(|s| process_text(&s)),
            owner: owner.id().to_owned(),
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

        Ok(id)
    }

    /// Get the entry with the specified id.
    pub fn entry(&self, id: String) -> Option<super::Entry> {
        if self.entries.contains_key(&id) {
            Some(super::Entry { index: self, id })
        } else {
            // See if any entries had the provided id as an old id.
            self.entries
                .values()
                .any(|e| e.old_ids.contains(&id))
                .then(|| super::Entry { index: self, id })
        }
    }

    /// Get the entry with the specified id for mutation.
    pub fn entry_mut(&mut self, id: String) -> Option<super::EntryMut> {
        if self.entries.contains_key(&id) {
            Some(super::EntryMut {
                index: self,
                id,
                exists: true,
            })
        } else {
            // See if any entries had the provided id as an old id.
            self.entries
                .values()
                .any(|e| e.old_ids.contains(&id))
                .then(|| super::EntryMut {
                    index: self,
                    id,
                    exists: true,
                })
        }
    }

    /// Add an entry.
    /// 
    /// Returns the entry's id.
    pub fn create_entry(
        &mut self,
        title: String,
        description: String,
        summary: String,
        author: User,
        position: Position<(Volume, usize), Entry>,
    ) -> Result<String, DuplicateIdError<String>> {
        // Make sure this id is not a duplicate.
        let id = create_id(&title);
        if self.entries.contains_key(&id) {
            return Err(DuplicateIdError(id));
        }

        // Get position.
        let (mut parent_volume, parent_volume_part, index_in_parent) = position.resolve(self);

        let entry = EntryData {
            title: process_text(&title),
            old_ids: Vec::new(),
            description: process_text(&description),
            summary: process_text(&summary),
            author: author.id().to_owned(),
            parent_volume: (parent_volume.id.to_owned(), parent_volume_part),
            sections: Vec::new(),
            search_index: search::Index::new(),
        };

        // Create files.
        File::create(format!("content/entries/{id}.json")).unwrap();
        File::create(format!("content/entries/{id}.index")).unwrap();

        // Insert into parent volume.
        parent_volume.data_mut().entries.insert(index_in_parent, id.clone());
        drop(parent_volume);
        
        // Insert into index.
        self.entries.insert(id.clone(), entry);

        Ok(id)
    }

    /// Get the section with the specified id.
    pub fn section(&self, id: u32) -> Option<super::Section> {
        self.sections
            .contains_key(&id)
            .then_some(super::Section { index: self, id })
    }

    /// Get the section with the specified id for mutation.
    pub fn section_mut(&mut self, id: u32) -> Option<super::SectionMut> {
        self.sections
            .contains_key(&id)
            .then_some(super::SectionMut {
                index: self,
                id,
                exists: true,
            })
    }

    /// Add a section.
    /// 
    /// Returns the section's id.
    pub fn create_section(
        &mut self,
        heading: Option<String>,
        description: String,
        summary: String,
        date: NaiveDate,
        position: Position<Entry, Section>,
    ) -> Result<u32, DuplicateIdError<u32>> {
        let id = self.next_section_id;
        self.next_section_id += 1;
        
        // Make sure this id is not a duplicate.
        if self.sections.contains_key(&id) {
            return Err(DuplicateIdError(id));
        }

        // Get position.
        let (mut parent_entry, index_in_parent) = position.resolve(self);

        let section = SectionData {
            heading: heading.map(|h| process_text(&h)),
            description: process_text(&description),
            summary: process_text(&summary),
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
        File::create(format!("content/section/{id}.txt")).unwrap();
        File::create(format!("content/section/{id}.json")).unwrap();
        File::create(format!("content/section/{id}.index")).unwrap();

        // Insert into parent entry.
        parent_entry.data_mut().sections.insert(index_in_parent, id);
        drop(parent_entry);
        
        // Insert into index.
        self.sections.insert(id, section);

        Ok(id)
    }
}
