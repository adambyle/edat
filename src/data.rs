use std::{collections::HashMap, fs::File};

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Index {
    users: HashMap<String, User>,
    volumes: HashMap<String, Volume>,
    entries: HashMap<String, Entry>,
    sections: HashMap<u32, Section>,
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
        let volumes: HashMap<String, Volume> = volumes
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

    pub fn sections(&self) -> impl Iterator<Item = SectionWrapper> {
        self.sections.iter().map(|(&k, s)| Wrapper::new(s, k))
    }

    pub fn section(&self, id: u32) -> Option<SectionWrapper> {
        self.sections.get(&id).map(|s| Wrapper::new(s, id))
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
}

impl User {
    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn has_code(&self, code: &str) -> bool {
        self.codes.iter().any(|c| c == code)
    }

    pub fn history(&self) -> Option<&[History]> {
        self.history.as_ref().map(|h| h.as_ref())
    }
}

impl Save for User {
    type Id = String;

    fn save(&self, id: &Self::Id) {
        let user = File::open(format!("users/{id}.json")).unwrap();
        serde_json::to_writer_pretty(user, self).unwrap();
    }

    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self> where Self: Sized {
        index.user_mut(&id).unwrap()
    }
}

pub type UserWrapperMut<'a> = WrapperMut<'a, User>;
pub type UserWrapper<'a> = Wrapper<'a, User>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserPrivilege {
    Owner,
    Reader,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct History {
    section: u32,
    progress: usize,
    timestamp: u64,
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

    pub fn content_type(&self) -> ContentType {
        self.content_type.clone()
    }

    pub fn entries<'a>(&'a self, index: &'a Index) -> impl Iterator<Item = EntryWrapper> {
        self.entries.iter().filter_map(|e| index.entry(e))
    }
}

impl Save for Volume {
    type Id = String;

    fn save(&self, id: &Self::Id) {
        let volume = File::open(format!("content/volumes/{id}.json")).unwrap();
        serde_json::to_writer_pretty(volume, self).unwrap();
    }

    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self> where Self: Sized {
        index.volume_mut(&id).unwrap()
    }
}

pub type VolumeWrapperMut<'a> = WrapperMut<'a, Volume>;
pub type VolumeWrapper<'a> = Wrapper<'a, Volume>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    parent_volume: (String, usize),
    author: String,
    summary: String,
    description: String,
    sections: Vec<u32>,
}

impl Entry {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }
}

impl Save for Entry {
    type Id = String;

    fn save(&self, id: &Self::Id) {
        let volume = File::open(format!("content/entries/{id}.json")).unwrap();
        serde_json::to_writer_pretty(volume, self).unwrap();
    }

    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self> where Self: Sized {
        index.entry_mut(&id).unwrap()
    }
}

pub type EntryWrapperMut<'a> = WrapperMut<'a, Entry>;
pub type EntryWrapper<'a> = Wrapper<'a, Entry>;

#[derive(Clone, Serialize, Deserialize)]
pub struct Section {
    parent_entry: String,
    status: ContentStatus,
    date: String,
    summary: String,
    description: String,
    comments: Vec<Comment>,
    perspectives: Vec<u32>,
}

impl Save for Section {
    type Id = u32;

    fn save(&self, id: &Self::Id) {
        let section = File::open(format!("content/sections/{id}.json")).unwrap();
        serde_json::to_writer_pretty(section, self).unwrap();
    }

    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self> where Self: Sized {
        index.section_mut(id).unwrap()
    }
}

pub type SectionWrapperMut<'a> = WrapperMut<'a, Section>;
pub type SectionWrapper<'a> = Wrapper<'a, Section>;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentStatus {
    Missing,
    Incomplete,
    Complete,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Comment {
    author: String,
    timestamp: u64,
    contents: String,
}

pub trait Save {
    type Id;

    fn save(&self, id: &Self::Id);
    
    fn get_mut<'b>(index: &'b mut Index, id: Self::Id) -> WrapperMut<'b, Self> where Self: Sized;
}

// pub trait Upgrade<I> : for<'a> Save<'a, Id = I> {

//     fn get_mut<'b>(index: &'b mut Index, id: I) -> WrapperMut<'b, Self>
//     where
//         Self: Sized + Save<'b>;
// }

pub struct Wrapper<'a, T>
where
    T: Save,
{
    id: T::Id,
    data: &'a T,
}

impl<'a, T> Wrapper<'a, T>
where
    T: Save,
{
    fn new(data: &'a T, id: T::Id) -> Self {
        Wrapper { id, data }
    }

    pub fn id(&self) -> &T::Id {
        &self.id
    }

    pub fn into_mut<'b>(self) -> IntoMut<T> {
        IntoMut(self.id)
    }
}

impl<'a, T> std::ops::Deref for Wrapper<'a, T>
where
    T: Save,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct WrapperMut<'a, T>
where
    T: Save,
{
    id: T::Id,
    data: &'a mut T,
    modified: bool,
}

impl<'a, T> WrapperMut<'a, T>
where
    T: Save,
{
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

impl<'a, T> std::ops::Deref for WrapperMut<'a, T>
where
    T: Save,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> std::ops::DerefMut for WrapperMut<'a, T>
where
    T: Save,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.modified = true;
        &mut self.data
    }
}

impl<'a, T> Drop for WrapperMut<'a, T>
where
    T: Save,
{
    fn drop(&mut self) {
        self.data.save(&self.id);
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
