use std::{collections::HashMap, fs};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::*;

use crate::search;

#[derive(Serialize, Deserialize)]
struct IndexFile {
    volumes: Vec<String>,
    next_section_id: u32,
}

pub struct Index {
    pub(super) users: HashMap<String, UserData>,
    pub(super) volumes: IndexMap<String, VolumeData>,
    pub(super) entries: HashMap<String, EntryData>,
    pub(super) sections: HashMap<u32, SectionData>,
    pub(super) next_section_id: u32,
}

impl Index {
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

    pub fn user(&self, id: String) -> Option<super::User> {
        self.users
            .contains_key(&id)
            .then_some(super::User { index: self, id })
    }

    pub fn user_mut(&mut self, id: String) -> Option<super::UserMut> {
        self.users
            .contains_key(&id)
            .then_some(super::UserMut { index: self, id })
    }

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

    pub fn section(&self, id: u32) -> Option<super::Section> {
        self.sections
            .contains_key(&id)
            .then_some(super::Section { index: self, id })
    }

    pub fn section_mut(&mut self, id: u32) -> Option<super::SectionMut> {
        self.sections
            .contains_key(&id)
            .then_some(super::SectionMut {
                index: self,
                id,
                exists: true,
            })
    }
}
