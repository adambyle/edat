use std::{
    collections::HashMap,
    fs::{remove_file, File},
    ops::Deref,
};

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Utc};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct IndexFile {
    volumes: Vec<String>,
    next_section_id: u32,
}

#[derive(Deserialize)]
pub enum Position<C, I> {
    StartOf(C),
    Before(I),
    After(I),
    EndOf(C),
}

#[derive(Debug)]
pub enum InvalidReference {
    Volume(String),
    Entry(String),
    Section(u32),
}

#[derive(Clone)]
pub struct Index {
    users: HashMap<String, User>,
    volumes: IndexMap<String, Volume>,
    entries: HashMap<String, Entry>,
    sections: HashMap<u32, Section>,
    next_section_id: u32,
}

impl Index {
    pub fn init() -> Self {
        let index_file = File::open("content/index.json").unwrap();
        let index_file: IndexFile = serde_json::from_reader(index_file).unwrap();

        let users = File::open("users/users.json").unwrap();
        let users: Vec<String> = serde_json::from_reader(users).unwrap();
        let users = users
            .into_iter()
            .map(|u| {
                let user = File::open(format!("users/{u}.json")).unwrap();
                let user = serde_json::from_reader(user).unwrap();
                (u, user)
            })
            .collect();

        let volumes: IndexMap<String, Volume> = index_file
            .volumes
            .into_iter()
            .map(|v| {
                let volume = File::open(format!("content/volumes/{v}.json")).unwrap();
                let volume = serde_json::from_reader(volume).unwrap();
                (v, volume)
            })
            .collect();

        let entries: HashMap<String, Entry> = volumes
            .values()
            .map(|v| v.entries.iter())
            .flatten()
            .map(|e| {
                let entry = File::open(format!("content/entries/{e}.json")).unwrap();
                let entry = serde_json::from_reader(entry).unwrap();
                (e.clone(), entry)
            })
            .collect();

        let sections = entries
            .values()
            .map(|e| e.sections.iter())
            .flatten()
            .map(|&s| {
                let section = File::open(format!("content/sections/{s}.json")).unwrap();
                let section = serde_json::from_reader(section).unwrap();
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

    fn save(&self) {
        let index_file = File::create("content/index.json").unwrap();
        serde_json::to_writer_pretty(
            index_file,
            &IndexFile {
                volumes: self.volumes.keys().map(|k| k.clone()).collect(),
                next_section_id: self.next_section_id,
            },
        )
        .unwrap();
    }

    pub fn next_section_id(&self) -> u32 {
        self.next_section_id
    }

    pub fn users(&self) -> impl Iterator<Item = UserWrapper> {
        self.users
            .iter()
            .map(|(k, u)| Wrapper::new(u, k.to_owned()))
    }

    pub fn user<'a>(&'a self, id: &str) -> Option<UserWrapper> {
        self.users.get(id).map(|u| Wrapper::new(u, id.to_owned()))
    }

    // pub fn users_mut(&mut self) -> impl Iterator<Item = UserWrapperMut> {
    //     self.users
    //         .iter_mut()
    //         .map(|(k, u)| WrapperMut::new(u, k.to_owned()))
    // }

    pub fn user_mut<'a>(&'a mut self, id: &str) -> Option<UserWrapperMut> {
        self.users
            .get_mut(id)
            .map(|u| WrapperMut::new(u, id.to_owned()))
    }

    pub fn remove_user(&mut self, id: &str) {
        if let Some(ref mut user) = self.user_mut(id) {
            user.codes.clear();
        }
    }

    pub fn set_user_name(
        &mut self,
        id: &str,
        first_name: String,
        last_name: String,
    ) -> Option<String> {
        let _ = remove_file(format!("users/{id}.json")).unwrap();

        let mut user = self.users.remove(id)?;
        let new_id = format!("{}{}", &first_name, &last_name);
        user.first_name = first_name;
        user.last_name = last_name;
        self.users.insert(new_id.clone(), user);

        for (_, volume) in &mut self.volumes {
            if volume.owner == id {
                volume.owner = new_id.clone();
            }
        }

        for (_, entry) in &mut self.entries {
            if entry.author == id {
                entry.author = new_id.clone();
            }
        }

        Some(new_id)
    }

    pub fn new_user(&mut self, first_name: String, last_name: String) {
        let id = format!("{}{}", first_name.to_lowercase(), last_name.to_lowercase());
        self.users.insert(
            id.clone(),
            User {
                first_name,
                last_name,
                privilege: UserPrivilege::Reader,
                codes: Vec::new(),
                widgets: Vec::new(),
                history: None,
                preferences: HashMap::new(),
            },
        );

        // Force a save.
        drop(self.user_mut(&id));
    }

    pub fn volumes(&self) -> impl Iterator<Item = VolumeWrapper> {
        self.volumes
            .iter()
            .map(|(k, v)| Wrapper::new(v, k.to_owned()))
    }

    pub fn volume<'a>(&'a self, id: &str) -> Option<VolumeWrapper> {
        self.volumes.get(id).map(|v| Wrapper::new(v, id.to_owned()))
    }

    // pub fn volumes_mut(&mut self) -> impl Iterator<Item = VolumeWrapperMut> {
    //     self.volumes
    //         .iter_mut()
    //         .map(|(k, v)| WrapperMut::new(v, k.to_owned()))
    // }

    pub fn volume_mut<'a>(&'a mut self, id: &str) -> Option<VolumeWrapperMut> {
        self.volumes
            .get_mut(id)
            .map(|v| WrapperMut::new(v, id.to_owned()))
    }

    pub fn remove_volume(&mut self, id: &str) {
        self.volumes.shift_remove(&id.to_owned());
        self.save();
    }

    pub fn new_volume(
        &mut self,
        title: String,
        subtitle: Option<String>,
        owner: String,
        position: Position<(), String>,
    ) -> Result<String, InvalidReference> {
        let id = create_id(&title);
        let subtitle = subtitle.map(|s| process_text(&s));
        let volume = Volume {
            title: process_text(&title),
            subtitle,
            owner,
            content_type: ContentType::Journal,
            volume_count: 1,
            entries: Vec::new(),
        };

        self.insert_volume(id.clone(), volume, position)?;
        Ok(id)
    }

    pub fn move_volume(
        &mut self,
        id: &str,
        position: Position<(), String>,
    ) -> Result<(), InvalidReference> {
        let volume = self
            .volumes
            .shift_remove(id)
            .ok_or(InvalidReference::Volume(id.to_owned()))?;
        self.insert_volume(id.to_owned(), volume, position)
    }

    fn insert_volume(
        &mut self,
        id: String,
        volume: Volume,
        position: Position<(), String>,
    ) -> Result<(), InvalidReference> {
        let index = match position {
            Position::StartOf(_) => 0,
            Position::EndOf(_) => self.volumes.len(),
            Position::After(reference) => {
                1 + self
                    .volumes
                    .keys()
                    .position(|v| v == &reference)
                    .ok_or(InvalidReference::Volume(reference))?
            }
            Position::Before(reference) => self
                .volumes
                .keys()
                .position(|v| v == &reference)
                .ok_or(InvalidReference::Volume(reference))?,
        };
        self.volumes.shift_insert(index, id.clone(), volume);

        // Force a save.
        drop(self.volume_mut(&id));

        Ok(())
    }

    pub fn set_volume_title(
        &mut self,
        id: &str,
        new_title: String,
    ) -> Result<String, InvalidReference> {
        let _ = remove_file(format!("content/volumes/{id}.json")).unwrap();

        // Rename and re-id volume.
        let index = self
            .volumes
            .keys()
            .position(|v| v == id)
            .ok_or(InvalidReference::Volume(id.to_owned()))?;
        let mut volume = self.volumes.shift_remove_index(index).unwrap().1;
        let new_id = create_id(&new_title);
        volume.title = process_text(&new_title);

        // Update child entries.
        for entry in &volume.entries {
            let mut entry = self.entry_mut(entry).unwrap();
            entry.parent_volume.0 = new_id.clone();
        }

        self.volumes.shift_insert(index, new_id.clone(), volume);
        self.save();
        Ok(new_id)
    }

    fn volume_recount(&mut self, id: &str) {
        let volume = self.volume(id).unwrap();
        let mut volume_count = 0;
        for entry in &volume.entries {
            let entry = self.entry(entry).unwrap();
            volume_count = volume_count.max(entry.parent_volume.1);
        }
        let mut volume = self.volume_mut(id).unwrap();
        volume.volume_count = volume_count;
    }

    pub fn entries(&self) -> impl Iterator<Item = EntryWrapper> {
        self.entries
            .iter()
            .map(|(k, e)| Wrapper::new(e, k.to_owned()))
    }

    pub fn entry<'a>(&'a self, id: &str) -> Option<EntryWrapper> {
        self.entries.get(id).map(|e| Wrapper::new(e, id.to_owned()))
    }

    // pub fn entries_mut(&mut self) -> impl Iterator<Item = EntryWrapperMut> {
    //     self.entries
    //         .iter_mut()
    //         .map(|(k, e)| WrapperMut::new(e, k.to_owned()))
    // }

    pub fn entry_mut<'a>(&'a mut self, id: &str) -> Option<EntryWrapperMut> {
        self.entries
            .get_mut(id)
            .map(|e| WrapperMut::new(e, id.to_owned()))
    }

    pub fn remove_entry(&mut self, id: &str) {
        if let Some(entry) = self.entry(id) {
            // Remove the section the parent volume's record.
            let parent_volume_id = entry.parent_volume.0.to_owned();
            {
                let mut parent_volume = self.volume_mut(&parent_volume_id).unwrap();
                parent_volume.entries.retain(|s| *s != id);
            }
            self.volume_recount(&parent_volume_id);

            // Remove the section from the record.
            self.entries.remove(id);
        }
    }

    pub fn new_entry(
        &mut self,
        title: String,
        description: String,
        summary: String,
        author: String,
        position: Position<(String, usize), String>,
    ) -> Result<String, InvalidReference> {
        let id = create_id(&title);
        let entry = Entry {
            title: process_text(&title),
            old_ids: Vec::new(),
            parent_volume: ("".to_owned(), 0),
            author,
            description: process_text(&description),
            summary: process_text(&summary),
            sections: Vec::new(),
        };

        self.insert_entry(id.clone(), entry, position)?;
        Ok(id)
    }

    pub fn move_entry(
        &mut self,
        id: &str,
        position: Position<(String, usize), String>,
    ) -> Result<(), InvalidReference> {
        let entry = self
            .entries
            .remove(id)
            .ok_or(InvalidReference::Entry(id.to_owned()))?;

        // Update parent volume entry list.
        let parent_volume_id = &entry.parent_volume.0;
        {
            let mut parent_volume = self.volume_mut(&parent_volume_id).unwrap();
            parent_volume.entries.retain(|e| e != id);
        }

        self.insert_entry(id.to_owned(), entry, position)
    }

    pub fn set_entry_title(
        &mut self,
        id: &str,
        new_title: String,
    ) -> Result<String, InvalidReference> {
        let _ = remove_file(format!("content/entries/{id}.json"));

        // Rename and re-id entry.
        let mut entry = self
            .entries
            .remove(id)
            .ok_or(InvalidReference::Entry(id.to_owned()))?;
        let new_id = create_id(&new_title);
        entry.title = process_text(&new_title);

        // Update child entries.
        for &section in &entry.sections {
            let mut section = self.section_mut(section).unwrap();
            section.parent_entry = new_id.clone();
        }

        self.entries.insert(new_id.clone(), entry);
        Ok(new_id)
    }

    fn insert_entry(
        &mut self,
        id: String,
        mut entry: Entry,
        position: Position<(String, usize), String>,
    ) -> Result<(), InvalidReference> {
        {
            let (mut volume, volume_part, index) = match position {
                Position::StartOf((volume_id, volume_part)) => {
                    let volume = self
                        .volume_mut(&volume_id)
                        .ok_or(InvalidReference::Volume(volume_id))?;
                    (volume, volume_part, 0)
                }
                Position::EndOf((volume_id, volume_part)) => {
                    let volume = self
                        .volume_mut(&volume_id)
                        .ok_or(InvalidReference::Volume(volume_id))?;
                    let volume_length = volume.entries.len();
                    (volume, volume_part, volume_length)
                }
                Position::After(reference) => {
                    let entry = self
                        .entry(&reference)
                        .ok_or(InvalidReference::Entry(reference.clone()))?;
                    let (ref volume, volume_part) = entry.parent_volume;
                    let volume = volume.clone();
                    let volume = self.volume_mut(&volume).unwrap();
                    let index = 1 + volume.entries.iter().position(|e| e == &reference).unwrap();
                    (volume, volume_part, index)
                }
                Position::Before(reference) => {
                    let entry = self
                        .entry(&reference)
                        .ok_or(InvalidReference::Entry(reference.clone()))?;
                    let (ref volume, volume_part) = entry.parent_volume;
                    let volume = volume.clone();
                    let volume = self.volume_mut(&volume).unwrap();
                    let index = volume.entries.iter().position(|e| e == &reference).unwrap();
                    (volume, volume_part, index)
                }
            };
            entry.parent_volume = (volume.id.clone(), volume_part);
            volume.volume_count = volume.volume_count.max(volume_part + 1);
            volume.entries.insert(index, id.clone());
        }

        self.entries.insert(id.clone(), entry);

        // Force a save.
        drop(self.entry_mut(&id));

        Ok(())
    }

    pub fn sections(&self) -> impl Iterator<Item = SectionWrapper> {
        self.sections.iter().map(|(&k, s)| Wrapper::new(s, k))
    }

    pub fn section(&self, id: u32) -> Option<SectionWrapper> {
        self.sections.get(&id).map(|s| Wrapper::new(s, id))
    }

    // pub fn sections_mut(&mut self) -> impl Iterator<Item = SectionWrapperMut> {
    //     self.sections
    //         .iter_mut()
    //         .map(|(&k, s)| WrapperMut::new(s, k))
    // }

    pub fn section_mut(&mut self, id: u32) -> Option<SectionWrapperMut> {
        self.sections.get_mut(&id).map(|s| WrapperMut::new(s, id))
    }

    pub fn remove_section(&mut self, id: u32) {
        if let Some(section) = self.section(id) {
            // Remove the section the parent entry's record.
            {
                let parent_entry_id = section.parent_entry.to_owned();
                let mut parent_entry = self.entry_mut(&parent_entry_id).unwrap();
                parent_entry.sections.retain(|s| *s != id);
            }

            // Remove the section from the record.
            self.sections.remove(&id);
        }
    }

    pub fn new_section(
        &mut self,
        heading: Option<String>,
        description: String,
        summary: String,
        date: NaiveDate,
        position: Position<String, u32>,
    ) -> Result<u32, InvalidReference> {
        let id = self.next_section_id;
        let heading = heading.map(|s| process_text(&s));
        self.next_section_id += 1;
        let section = Section {
            description: process_text(&description),
            summary: process_text(&summary),
            heading,
            parent_entry: "".to_owned(),
            status: ContentStatus::Missing,
            date: date.format("%Y-%m-%d").to_string(),
            comments: Vec::new(),
            perspectives: Vec::new(),
            length: 0,
        };

        self.insert_section(id, section, position)?;
        File::create(format!("content/sections/{id}.txt")).unwrap();
        self.save();
        Ok(id)
    }

    pub fn move_section(
        &mut self,
        id: u32,
        position: Position<String, u32>,
    ) -> Result<(), InvalidReference> {
        let section = self
            .sections
            .remove(&id)
            .ok_or(InvalidReference::Section(id))?;

        // Update parent entry section list.
        let parent_entry_id = &section.parent_entry;
        {
            let mut parent_entry = self.entry_mut(&parent_entry_id).unwrap();
            parent_entry.sections.retain(|s| *s != id);
        }

        self.insert_section(id, section, position)
    }

    fn insert_section(
        &mut self,
        id: u32,
        mut section: Section,
        position: Position<String, u32>,
    ) -> Result<(), InvalidReference> {
        {
            let (mut entry, index) = match position {
                Position::StartOf(entry_id) => {
                    let entry = self
                        .entry_mut(&entry_id)
                        .ok_or(InvalidReference::Entry(entry_id))?;
                    (entry, 0)
                }
                Position::EndOf(entry_id) => {
                    let entry = self
                        .entry_mut(&entry_id)
                        .ok_or(InvalidReference::Entry(entry_id))?;
                    let entry_length = entry.sections.len();
                    (entry, entry_length)
                }
                Position::After(reference) => {
                    let section = self
                        .section(reference)
                        .ok_or(InvalidReference::Section(reference))?;
                    let entry = section.parent_entry.clone();
                    let entry = self.entry_mut(&entry).unwrap();
                    let index = 1 + entry.sections.iter().position(|e| e == &reference).unwrap();
                    (entry, index)
                }
                Position::Before(reference) => {
                    let section = self
                        .section(reference)
                        .ok_or(InvalidReference::Section(reference))?;
                    let entry = section.parent_entry.clone();
                    let entry = self.entry_mut(&entry).unwrap();
                    let index = entry.sections.iter().position(|e| e == &reference).unwrap();
                    (entry, index)
                }
            };
            section.parent_entry = entry.id.clone();
            entry.sections.insert(index, id);
        }
        self.sections.insert(id, section);

        // Force a save.
        drop(self.section_mut(id));

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    first_name: String,
    last_name: String,
    privilege: UserPrivilege,
    codes: Vec<String>,
    widgets: Vec<String>,
    history: Option<Vec<History>>,
    preferences: HashMap<String, String>,
}

impl User {
    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn privilege(&self) -> UserPrivilege {
        self.privilege.clone()
    }

    pub fn set_privilege(&mut self, privilege: UserPrivilege) {
        self.privilege = privilege;
    }

    pub fn has_code(&self, code: &str) -> bool {
        self.codes.iter().any(|c| c == code)
    }

    pub fn codes(&self) -> &[String] {
        &self.codes
    }

    pub fn add_code(&mut self, code: String) {
        if !self.codes.contains(&code) {
            self.codes.push(code);
        }
    }

    pub fn remove_code(&mut self, code: &str) {
        self.codes.retain(|c| c != code);
    }

    pub fn history(&self) -> Option<&[History]> {
        self.history.as_ref().map(|h| h.as_ref())
    }

    pub fn read_section(&mut self, section: u32, progress: usize, finished: bool) {
        let now = Utc::now().timestamp();
        let history = self.init_history();

        // Update the timestamp on a section if it is already present
        // in the user's history.
        match history.iter_mut().find(|h| h.section == section) {
            Some(section) => {
                section.ever_finished |= finished;
                section.progress = progress;
                section.timestamp = now;
            }
            None => {
                history.push(History {
                    section,
                    progress,
                    timestamp: now,
                    ever_finished: finished,
                });
            }
        }
    }

    pub fn has_read_section(&self, section: u32, must_have_finished: bool) -> bool {
        let Some(ref history) = self.history else {
            return false;
        };
        history
            .iter()
            .any(|h| h.section == section && (h.ever_finished || !must_have_finished))
    }

    pub fn has_started_entry(&self, entry: &str, index: &Index) -> bool {
        let Some(ref history) = self.history else {
            return false;
        };
        history.iter().any(|h| {
            let Some(section) = index.section(h.section) else {
                return false;
            };
            section.parent_entry == entry
        })
    }

    pub fn has_read_entry(&self, entry: &str, index: &Index, must_have_finished: bool) -> bool {
        let Some(ref history) = self.history else {
            return false;
        };
        let Some(entry) = index.entry(entry) else {
            return false;
        };
        let mut sections = entry.sections(index);
        sections.all(|s| {
            history
                .iter()
                .any(|h| h.section == *s.id() && (h.ever_finished || !must_have_finished))
        })
    }

    pub fn empty_history(&mut self) {
        self.history = Some(Vec::new());
    }

    pub fn widgets(&self) -> &[String] {
        &self.widgets
    }

    pub fn set_widgets(&mut self, widgets: Vec<String>) {
        self.widgets = widgets;
    }

    pub fn preferences(&self) -> &HashMap<String, String> {
        &self.preferences
    }

    pub fn preferences_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.preferences
    }

    fn init_history(&mut self) -> &mut Vec<History> {
        match self.history {
            Some(ref mut history) => history,
            None => {
                self.history = Some(Vec::new());
                self.history.as_mut().unwrap()
            }
        }
    }
}

impl Save for User {
    type Id = String;

    fn save(&self, id: &Self::Id) {
        let user = File::create(format!("users/{id}.json")).unwrap();
        serde_json::to_writer_pretty(user, self).unwrap();
    }
}

pub type UserWrapperMut<'a> = WrapperMut<'a, User>;
pub type UserWrapper<'a> = Wrapper<'a, User>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum UserPrivilege {
    Owner,
    Reader,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct History {
    section: u32,
    progress: usize,
    timestamp: i64,
    ever_finished: bool,
}

impl History {
    pub fn section_id(&self) -> u32 {
        self.section
    }

    pub fn progress(&self) -> usize {
        self.progress
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.timestamp, 0).unwrap()
    }

    pub fn ever_finished(&self) -> bool {
        self.ever_finished
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Volume {
    title: String,
    subtitle: Option<String>,
    owner: String,
    content_type: ContentType,
    volume_count: usize,
    entries: Vec<String>,
}

impl Volume {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn subtitle(&self) -> Option<&str> {
        self.subtitle.as_ref().map(|x| x.as_str())
    }

    pub fn set_subtitle(&mut self, subtitle: Option<String>) {
        self.subtitle = subtitle.map(|s| process_text(&s));
    }

    pub fn owner_id(&self) -> &str {
        &self.owner
    }

    pub fn content_type(&self) -> ContentType {
        self.content_type.clone()
    }

    pub fn set_content_type(&mut self, content_type: ContentType) {
        self.content_type = content_type
    }

    pub fn entries<'a>(&'a self, index: &'a Index) -> impl Iterator<Item = EntryWrapper> {
        self.entries.iter().filter_map(|e| index.entry(e))
    }

    pub fn volume_count(&self) -> usize {
        self.volume_count
    }
}

impl Save for Volume {
    type Id = String;

    fn save(&self, id: &Self::Id) {
        let volume = File::create(format!("content/volumes/{id}.json")).unwrap();
        serde_json::to_writer_pretty(volume, self).unwrap();
    }
}

pub type VolumeWrapperMut<'a> = WrapperMut<'a, Volume>;
pub type VolumeWrapper<'a> = Wrapper<'a, Volume>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ContentType {
    Journal,
    Archive,
    Diary,
    Cartoons,
    Creative,
    Featured,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    title: String,
    old_ids: Vec<String>,
    parent_volume: (String, usize),
    author: String,
    description: String,
    summary: String,
    sections: Vec<u32>,
}

impl Entry {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn parent_volume_id(&self) -> &str {
        &self.parent_volume.0
    }

    pub fn parent_volume_index(&self) -> usize {
        self.parent_volume.1
    }

    pub fn author_id(&self) -> &str {
        &self.author
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.description = process_text(&description);
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn set_summary(&mut self, summary: String) {
        self.summary = process_text(&summary);
    }

    pub fn section_ids(&self) -> &[u32] {
        &self.sections
    }

    pub fn sections<'a>(&'a self, index: &'a Index) -> impl Iterator<Item = SectionWrapper<'a>> {
        self.sections.iter().filter_map(|s| index.section(*s))
    }
}

impl Save for Entry {
    type Id = String;

    fn save(&self, id: &Self::Id) {
        let volume = File::create(format!("content/entries/{id}.json")).unwrap();
        serde_json::to_writer_pretty(volume, self).unwrap();
    }
}

pub type EntryWrapperMut<'a> = WrapperMut<'a, Entry>;
pub type EntryWrapper<'a> = Wrapper<'a, Entry>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Section {
    heading: Option<String>,
    parent_entry: String,
    status: ContentStatus,
    date: String,
    description: String,
    summary: String,
    comments: Vec<Comment>,
    perspectives: Vec<u32>,
    length: usize,
}

impl Section {
    pub fn heading(&self) -> Option<&str> {
        self.heading.as_ref().map(|h| h.as_ref())
    }

    pub fn set_heading(&mut self, heading: Option<String>) {
        self.heading = heading.map(|h| process_text(&h));
    }

    pub fn parent_entry_id(&self) -> &str {
        &self.parent_entry
    }

    pub fn status(&self) -> ContentStatus {
        self.status.clone()
    }

    pub fn set_status(&mut self, status: ContentStatus) {
        self.status = status;
    }

    pub fn raw_date(&self) -> &str {
        &self.date
    }

    pub fn date(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").unwrap()
    }

    pub fn set_date(&mut self, date: NaiveDate) {
        self.date = date.format("%Y-%m-%d").to_string();
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn set_description(&mut self, description: String) {
        self.description = process_text(&description);
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn set_summary(&mut self, summary: String) {
        self.summary = process_text(&summary);
    }

    pub fn comments(&self) -> &[Comment] {
        &self.comments
    }

    pub fn perspective_ids(&self) -> &[u32] {
        &self.perspectives
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn length_str(&self) -> String {
        if self.length < 2000 {
            (self.length / 10 * 10).to_string()
        } else {
            format!("{:.1}k", self.length / 1000)
        }
    }
}

impl Save for Section {
    type Id = u32;

    fn save(&self, id: &Self::Id) {
        let section = File::create(format!("content/sections/{id}.json")).unwrap();
        serde_json::to_writer_pretty(section, self).unwrap();
    }
}

pub type SectionWrapperMut<'a> = WrapperMut<'a, Section>;
pub type SectionWrapper<'a> = Wrapper<'a, Section>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ContentStatus {
    Missing,
    Incomplete,
    Complete,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Comment {
    author: String,
    timestamp: i64,
    contents: String,
}

impl Comment {
    pub fn author_id(&self) -> &str {
        &self.author
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.timestamp, 0).unwrap()
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }
}

pub trait Save {
    type Id;

    fn save(&self, id: &Self::Id);
}

pub struct Wrapper<'a, T: Save> {
    id: T::Id,
    data: &'a T,
}

impl<'a, T: Save> Wrapper<'a, T> {
    fn new(data: &'a T, id: T::Id) -> Self {
        Wrapper { id, data }
    }

    pub fn id(&self) -> &T::Id {
        &self.id
    }
}

impl<'a, T: Save> Deref for Wrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct WrapperMut<'a, T: Save> {
    id: T::Id,
    data: &'a mut T,
    modified: bool,
}

impl<'a, T: Save> WrapperMut<'a, T> {
    fn new(data: &'a mut T, id: T::Id) -> Self {
        WrapperMut {
            id,
            data,
            modified: false,
        }
    }

    pub fn id(&self) -> &T::Id {
        &self.id
    }
}

impl<'a, T: Save> Deref for WrapperMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T: Save> std::ops::DerefMut for WrapperMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.modified = true;
        &mut self.data
    }
}

impl<'a, T: Save> Drop for WrapperMut<'a, T> {
    fn drop(&mut self) {
        self.data.save(&self.id);
    }
}

pub fn date_naive(date: &NaiveDate) -> String {
    let now = Utc::now();
    if now.year_ce() == date.year_ce() {
        date.format("%b %-d").to_string()
    } else {
        date.format("%b %-d, %Y").to_string()
    }
}

pub fn date(date: &DateTime<Utc>) -> String {
    let now = Utc::now();
    if now.year_ce() == date.year_ce() {
        date.format("%b %-d").to_string()
    } else {
        date.format("%b %-d, %Y").to_string()
    }
}

pub fn roman_numeral(number: usize) -> &'static str {
    match number {
        1 => "I",
        2 => "II",
        3 => "III",
        _ => "?",
    }
}

pub fn create_id(name: &str) -> String {
    let name: String = name
        .replace("<i>", "")
        .replace("&", "and")
        .replace("</i>", "")
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
        .collect();
    name.replace(' ', "-")
}

fn process_text(text: &str) -> String {
    let text = text
        .replace("--", "—")
        .replace("-.", "–")
        .replace("...", "…");

    let open_quote = Regex::new(r#""\S"#).unwrap();
    let text = open_quote.replace_all(&text, "“");

    let quote = Regex::new(r#"""#).unwrap();
    let text = quote.replace_all(&text, "”");

    let open_single = Regex::new(r#"(\s')|(^')|(["“”]')"#).unwrap();
    let text = open_single.replace_all(&text, "‘");

    let quote = Regex::new(r#"'"#).unwrap();
    let text = quote.replace_all(&text, "’");

    text.into_owned()
}
