use std::{collections::HashMap, fs::File, ops::Deref};

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Index {
    users: IndexMap<String, User>,
    volumes: IndexMap<String, Volume>,
    entries: IndexMap<String, Entry>,
    sections: IndexMap<u32, Section>,
    next_section_id: u32,
}

impl Index {
    pub fn init() -> Self {
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

        let volumes = File::open("content/volumes.json").unwrap();
        let volumes: Vec<String> = serde_json::from_reader(volumes).unwrap();
        let volumes: IndexMap<String, Volume> = volumes
            .into_iter()
            .map(|v| {
                let volume = File::open(format!("content/volumes/{v}.json")).unwrap();
                let volume = serde_json::from_reader(volume).unwrap();
                (v, volume)
            })
            .collect();

        let entries: IndexMap<String, Entry> = volumes
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

        #[derive(Deserialize)]
        struct IndexFile {
            next_section_id: u32,
        }

        let index_file = File::open("content/index.json").unwrap();
        let index_file: IndexFile = serde_json::from_reader(index_file).unwrap();

        Index {
            users,
            volumes,
            entries,
            sections,
            next_section_id: index_file.next_section_id,
        }
    }

    pub fn users(&self) -> impl Iterator<Item = UserWrapper> {
        self.users
            .iter()
            .map(|(k, u)| Wrapper::new(u, k.to_owned()))
    }

    pub fn user<'a>(&'a self, id: &str) -> Option<UserWrapper> {
        self.users.get(id).map(|u| Wrapper::new(u, id.to_owned()))
    }

    pub fn volumes(&self) -> impl Iterator<Item = VolumeWrapper> {
        self.volumes
            .iter()
            .map(|(k, v)| Wrapper::new(v, k.to_owned()))
    }

    pub fn volume<'a>(&'a self, id: &str) -> Option<VolumeWrapper> {
        self.volumes.get(id).map(|v| Wrapper::new(v, id.to_owned()))
    }

    pub fn entries(&self) -> impl Iterator<Item = EntryWrapper> {
        self.entries
            .iter()
            .map(|(k, e)| Wrapper::new(e, k.to_owned()))
    }

    pub fn entry<'a>(&'a self, id: &str) -> Option<EntryWrapper> {
        self.entries.get(id).map(|e| Wrapper::new(e, id.to_owned()))
    }

    pub fn remove_entry(&mut self, id: &str) {
        if let Some(entry) = self.entry(id) {
            let parent_volume = entry.parent_volume_id().to_owned();
            let parent_volume = &mut self.volumes[&parent_volume];
            let i = parent_volume.entries.iter().position(|e| e == id).unwrap();
            parent_volume.entries.remove(i);
            self.entries.shift_remove(id);
        }
    }

    pub fn sections(&self) -> impl Iterator<Item = SectionWrapper> {
        self.sections.iter().map(|(&k, s)| Wrapper::new(s, k))
    }

    pub fn section(&self, id: u32) -> Option<SectionWrapper> {
        self.sections.get(&id).map(|s| Wrapper::new(s, id))
    }

    pub fn remove_section(&mut self, id: u32) {
        if let Some(section) = self.section(id) {
            let parent_entry = section.parent_entry_id().to_owned();
            let parent_entry = &mut self.entries[&parent_entry];
            let i = parent_entry.sections.iter().position(|s| *s == id).unwrap();
            parent_entry.sections.remove(i);
            self.sections.shift_remove(&id);
        }
    }

    pub fn users_mut(&mut self) -> impl Iterator<Item = UserWrapperMut> {
        self.users
            .iter_mut()
            .map(|(k, u)| WrapperMut::new(u, k.to_owned()))
    }

    pub fn user_mut<'a>(&'a mut self, id: &str) -> Option<UserWrapperMut> {
        self.users
            .get_mut(id)
            .map(|u| WrapperMut::new(u, id.to_owned()))
    }

    pub fn volumes_mut(&mut self) -> impl Iterator<Item = VolumeWrapperMut> {
        self.volumes
            .iter_mut()
            .map(|(k, v)| WrapperMut::new(v, k.to_owned()))
    }

    pub fn volume_mut<'a>(&'a mut self, id: &str) -> Option<VolumeWrapperMut> {
        self.volumes
            .get_mut(id)
            .map(|v| WrapperMut::new(v, id.to_owned()))
    }

    pub fn entries_mut(&mut self) -> impl Iterator<Item = EntryWrapperMut> {
        self.entries
            .iter_mut()
            .map(|(k, e)| WrapperMut::new(e, k.to_owned()))
    }

    pub fn entry_mut<'a>(&'a mut self, id: &str) -> Option<EntryWrapperMut> {
        self.entries
            .get_mut(id)
            .map(|e| WrapperMut::new(e, id.to_owned()))
    }

    pub fn sections_mut(&mut self) -> impl Iterator<Item = SectionWrapperMut> {
        self.sections
            .iter_mut()
            .map(|(&k, s)| WrapperMut::new(s, k))
    }

    pub fn section_mut(&mut self, id: u32) -> Option<SectionWrapperMut> {
        self.sections.get_mut(&id).map(|s| WrapperMut::new(s, id))
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
        if let Some(i) = self.codes.iter().position(|c| c == code) {
            self.codes.remove(i);
        }
    }

    pub fn history(&self) -> Option<&[History]> {
        self.history.as_ref().map(|h| h.as_ref())
    }

    pub fn read_section(&mut self, section: u32, progress: usize, finished: bool) {
        let now = Utc::now().timestamp();
        let history = self.init_history();
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

    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self>
    where
        Self: Sized,
    {
        index.user_mut(&id).unwrap()
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

    pub fn owner_id(&self) -> &str {
        &self.owner
    }

    pub fn content_type(&self) -> ContentType {
        self.content_type.clone()
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

    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self>
    where
        Self: Sized,
    {
        index.volume_mut(&id).unwrap()
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

    pub fn parent_volume<'a>(&self, index: &'a Index) -> VolumeWrapper<'a> {
        index.volume(&self.parent_volume.0).unwrap()
    }

    pub fn author_id(&self) -> &str {
        &self.author
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn section_ids(&self) -> &[u32] {
        &self.sections
    }

    pub fn sections<'a>(&'a self, index: &'a Index) -> impl Iterator<Item = SectionWrapper> {
        self.sections.iter().filter_map(|s| index.section(*s))
    }
}

impl Save for Entry {
    type Id = String;

    fn save(&self, id: &Self::Id) {
        let volume = File::create(format!("content/entries/{id}.json")).unwrap();
        serde_json::to_writer_pretty(volume, self).unwrap();
    }

    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self>
    where
        Self: Sized,
    {
        index.entry_mut(&id).unwrap()
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
        self.heading = heading;
    }

    pub fn parent_entry_id(&self) -> &str {
        &self.parent_entry
    }

    pub fn parent_entry<'a>(&self, index: &'a Index) -> EntryWrapper<'a> {
        index.entry(&self.parent_entry).unwrap()
    }

    pub fn status(&self) -> ContentStatus {
        self.status.clone()
    }

    pub fn date(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").unwrap()
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn description(&self) -> &str {
        &self.description
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

    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self>
    where
        Self: Sized,
    {
        index.section_mut(id).unwrap()
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

    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self>
    where
        Self: Sized;
}

pub trait AnyWrapper<T: Save>: Deref<Target = T> {
    fn id(&self) -> &T::Id;
}

pub struct Wrapper<'a, T: Save> {
    id: T::Id,
    data: &'a T,
}

impl<'a, T: Save> Wrapper<'a, T> {
    fn new(data: &'a T, id: T::Id) -> Self {
        Wrapper { id, data }
    }

    pub fn into_mut<'b>(self) -> IntoMut<T> {
        IntoMut(self.id)
    }
}

impl<T: Save> AnyWrapper<T> for Wrapper<'_, T> {
    fn id(&self) -> &T::Id {
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

impl<T: Save> AnyWrapper<T> for WrapperMut<'_, T> {
    fn id(&self) -> &T::Id {
        &self.id
    }
}

pub struct IntoMut<T: Save>(T::Id);

impl<T: Save> IntoMut<T> {
    pub fn resolve<'b>(self, index: &'b mut Index) -> WrapperMut<'b, T> {
        T::get_mut(index, self.0)
    }
}

#[macro_export]
macro_rules! upgrade {
    ($data:ident, $index:ident) => {
        let $data = $data.into_mut().resolve(&mut $index);
    };
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
