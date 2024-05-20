use std::{any::Any, collections::HashMap, fs::File, hash::Hash};

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use serde_json::Value;

pub fn load_users() -> Vec<User> {
    let users_file = File::open("users/users.json").expect("failed to open users.json");
    let doc: Value = serde_json::from_reader(users_file).expect("failed to parse users.json");

    doc["users"]
        .as_array()
        .unwrap()
        .into_iter()
        .map(User::from_json)
        .collect()
}

pub fn load_content() -> Content {
    let content_file = File::open("content/index.json").expect("failed to open index.json");
    let doc: Value = serde_json::from_reader(content_file).expect("failed to parse index.json");

    let volumes: HashMap<String, Volume> = doc["volumes"]
        .as_array()
        .unwrap()
        .into_iter()
        .map(|v| {
            let volume = v.as_str().unwrap();
            let volume_file = File::open(format!("content/volumes/{volume}.json")).expect("failed to open volume: {volume}");
            let doc = serde_json::from_reader(volume_file).expect("failed to parse volume");
            let volume = Volume::from_json(volume.to_owned(), &doc);
            (volume.id.clone(), volume)
        })
        .collect();

    let entries: HashMap<String, Entry> = volumes.values().map(|v| v.entries.iter()).flatten().map(|e| {
        let entry_file = File::open(format!("content/entries/{e}.json")).expect("failed to open entry");
        let doc = serde_json::from_reader(entry_file).expect("failed to parse entry");
        let entry = Entry::from_json(e.clone(), &doc);
        (entry.id.clone(), entry)
    }).collect();

    let sections = entries.values().map(|e| e.sections.iter()).flatten().map(|s| {
        let section_file = File::open(format!("content/sections/{s}.json")).expect("failed to open section");
        let doc: Value = serde_json::from_reader(section_file).expect("failed to parse section");
        let section = Section::from_json(*s, &doc);
        (*s, section)
    }).collect();

    Content {
        volumes,
        entries,
        sections,
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum UserPrivilege {
    Owner,
    Reader,
}

#[derive(Clone, Eq)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub privilege: UserPrivilege,
    pub codes: Vec<String>,
    pub entries_read: Vec<(String, NaiveDateTime)>,
    pub sections_read: Vec<(u64, NaiveDateTime)>,
}

impl User {
    pub fn full_name(&self) -> String {
        format!("{}{}", self.first_name, self.last_name).to_lowercase()
    }

    pub fn from_json(json: &Value) -> User {
        let first_name = json["first_name"].as_str().unwrap().to_owned();
        let last_name = json["last_name"].as_str().unwrap().to_owned();
        let privilege = match json["privilege"].as_str().unwrap() {
            "owner" => UserPrivilege::Owner,
            "reader" => UserPrivilege::Reader,
            other => panic!("invalid user privilege: {other}"),
        };
        let codes = json["codes"]
            .as_array()
            .unwrap()
            .into_iter()
            .map(|code| code.as_str().unwrap().to_owned())
            .collect();
        let entries_read = json["entries_read"]
            .as_array()
            .unwrap()
            .into_iter()
            .map(|entry| {
                (
                    entry[0].as_str().unwrap().to_owned(),
                    NaiveDateTime::parse_from_str(entry[1].as_str().unwrap(), "%F").unwrap(),
                )
            })
            .collect();
        let sections_read = json["sections_read"]
            .as_array()
            .unwrap()
            .into_iter()
            .map(|entry| {
                (
                    entry[0].as_u64().unwrap(),
                    NaiveDateTime::parse_from_str(entry[1].as_str().unwrap(), "%F").unwrap(),
                )
            })
            .collect();
        User {
            first_name,
            last_name,
            privilege,
            codes,
            entries_read,
            sections_read,
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.first_name == other.first_name && self.last_name == other.last_name
    }
}

fn get_user<'a>(users: &'a [User], name: &str) -> Option<&'a User> {
    users.iter().find(|user| user.full_name() == name)
}

#[derive(Clone)]
pub struct Content {
    pub volumes: HashMap<String, Volume>,
    pub entries: HashMap<String, Entry>,
    pub sections: HashMap<u32, Section>,
}

#[derive(Clone)]
pub enum VolumeType {
    Journal,
    Archive,
    Creative,
    Cartoons,
    Featured,
}

#[derive(Clone)]
pub struct Volume {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    owner: String,
    pub content_type: VolumeType,
    pub volume_count: usize,
    pub entries: Vec<String>,
}

impl Volume {
    pub fn owner<'a>(&self, users: &'a [User]) -> Option<&'a User> {
        get_user(users, &self.owner)
    }

    pub fn from_json(id: String, json: &Value) -> Volume {
        let title = json["title"].as_str().unwrap().to_owned();
        let subtitle = json.get("subtitle").map(|s| s.as_str().unwrap().to_owned());
        let owner = json["owner"].as_str().unwrap().to_owned();
        let content_type = match json["type"].as_str().unwrap() {
            "journal" => VolumeType::Journal,
            "archive" => VolumeType::Archive,
            "creative" => VolumeType::Creative,
            "cartoons" => VolumeType::Cartoons,
            "featured" => VolumeType::Featured,
            other => panic!("invalid volume type: {other}"),
        };
        let volume_count = json["volumes"].as_u64().unwrap() as usize;
        let entries = json["entries"]
            .as_array()
            .unwrap()
            .into_iter()
            .map(|e| e.as_str().unwrap().to_owned())
            .collect();

        Volume {
            id,
            title,
            subtitle,
            owner,
            content_type,
            volume_count,
            entries,
        }
    }
}

#[derive(Clone)]
pub struct Entry {
    pub id: String,
    pub name: String,
    parent_volume: (String, usize),
    author: String,
    pub description: String,
    pub sections: Vec<u32>,
}

impl Entry {
    pub fn parent_volume<'a>(&self, index: &'a Content) -> Option<(&'a Volume, usize)> {
        index
            .volumes
            .get(&self.parent_volume.0)
            .map(|v| (v, self.parent_volume.1))
    }

    pub fn author<'a>(&self, users: &'a [User]) -> Option<&'a User> {
        get_user(users, &self.author)
    }

    pub fn from_json(id: String, json: &Value) -> Entry {
        let name = json["name"].as_str().unwrap().to_owned();
        let parent_volume = json["parent_volume"].as_array().unwrap();
        let parent_volume = (
            parent_volume[0].as_str().unwrap().to_owned(),
            parent_volume[1].as_u64().unwrap() as usize,
        );
        let author = json["author"].as_str().unwrap().to_owned();
        let description = json["description"].as_str().unwrap().to_owned();
        let sections = json["sections"]
            .as_array()
            .unwrap()
            .into_iter()
            .map(|s| s.as_u64().unwrap() as u32)
            .collect();

        Entry {
            id,
            name,
            parent_volume,
            author,
            description,
            sections,
        }
    }
}

#[derive(Clone)]
pub enum Completion {
    Missing,
    Incomplete,
    Complete,
}

#[derive(Clone)]
pub enum Timestamp {
    Date(NaiveDateTime),
    Custom(String),
}

#[derive(Clone)]
pub struct Section {
    pub id: u32,
    parent_entry: String,
    pub status: Completion,
    pub date: Timestamp,
    pub description: Option<String>,
    pub perspectives: Vec<u32>,
    pub comments: Vec<Comment>,
}

impl Section {
    pub fn parent_volume<'a>(&self, index: &'a Content) -> Option<&'a Entry> {
        index.entries.get(&self.parent_entry)
    }

    pub fn from_json(id: u32, json: &Value) -> Section {
        let parent_entry = json["parent_entry"].as_str().unwrap().to_owned();
        let status = match json["status"].as_str().unwrap() {
            "missing" => Completion::Missing,
            "incomplete" => Completion::Incomplete,
            "complete" => Completion::Complete,
            other => panic!("invalid completion status: {other}"),
        };
        let date = json["date"].as_str().unwrap();
        let date = match NaiveDateTime::parse_from_str(date, "%F") {
            Ok(date) => Timestamp::Date(date),
            Err(_) => Timestamp::Custom(date.to_owned()),
        };
        let description = json
            .get("description")
            .map(|d| d.as_str().unwrap().to_owned());
        let comments = json["comments"]
            .as_array()
            .unwrap()
            .into_iter()
            .map(Comment::from_json)
            .collect();
        let perspectives = json["perspectives"]
            .as_array()
            .unwrap()
            .into_iter()
            .map(|p| p.as_u64().unwrap() as u32)
            .collect();

        Section {
            id,
            parent_entry,
            status,
            date,
            description,
            perspectives,
            comments,
        }
    }
}

#[derive(Clone)]
pub struct Comment {
    pub paragraph: usize,
    author: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
}

impl Comment {
    pub fn author<'a>(&self, users: &'a [User]) -> Option<&'a User> {
        get_user(users, &self.author)
    }

    pub fn from_json(json: &Value) -> Comment {
        let paragraph = json["paragraph"].as_u64().unwrap() as usize;
        let author = json["author"].as_str().unwrap().to_owned();
        let timestamp =
            DateTime::from_timestamp_millis(json["timestamp"].as_i64().unwrap()).unwrap();
        let content = json["content"].as_str().unwrap().to_owned();

        Comment {
            paragraph,
            author,
            timestamp,
            content,
        }
    }
}
