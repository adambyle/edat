use super::*;

pub struct UserInfo {
    pub first_name: String,
    pub last_name: String,
    pub privilege: String,
    pub codes: String,
    pub widgets: String,
    pub history: Vec<UserHistoryEntry>,
    pub preferences: Vec<UserPreference>,
}

pub struct UserHistoryEntry {
    pub entry: String,
    pub timestamp: i64,
}

pub struct UserPreference {
    pub setting: String,
    pub switch: String,
}

pub struct VolumeInfo {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub owner: String,
    pub content_type: String,
    pub volume_count: usize,
    pub entries: Vec<VolumeEntry>,
}

pub struct VolumeEntry {
    pub id: String,
    pub description: String,
}

pub struct EntryInfo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub summary: String,
    pub parent_volume: (String, usize),
    pub author: String,
    pub sections: Vec<EntrySection>,
}

pub struct EntrySection {
    pub id: u32,
    pub description: String,
}

pub struct SectionInfo {
    pub id: u32,
    pub heading: String,
    pub description: String,
    pub summary: String,
    pub parent_entry: String,
    pub status: String,
    pub date: String,
    pub in_entry: (usize, usize),
    pub length: usize,
    pub perspectives: String,
    pub comments: Vec<SectionComment>,
}

pub struct SectionComment {
    pub author: String,
    pub timestamp: i64,
    pub contents: String,
}

pub struct Volumes(pub Vec<(String, String)>);

pub fn missing(category: &str, id: String) -> maud::Markup {
    html! {
        p.error { "Unknown " (category) " " mono { (id) } }
    }
}

pub fn duplicate(id: String) -> maud::Markup {
    html! {
        p.error { "Taken id " mono { (id) } }
    }
}

pub fn bad_date(date: &str) -> maud::Markup {
    html! {
        p.error { "Invalid date " (date) }
    }
}

pub fn unauthorized() -> maud::Markup {
    html! {
        p.error { "Not authorized" }
    }
}

pub fn image_error(id: &str) -> maud::Markup {
    html! {
        p.error { "Failed to upload image: " (id) }
    }
}

pub fn image_success(id: &str) -> maud::Markup {
    html! {
        p { "Image uploaded: " (id) }
    }
}

pub fn images() -> maud::Markup {
    html! {
        p { b { "Images console" } }
        label { "Enter image ID to load" }
        input #image-id type="text";
        img #image;
        p { "Upload here" }
        input #image-upload type="file" multiple;
        button #upload { "Upload" }
        #image-feedback {}
    }
}

pub fn content(id: String, contents: String) -> maud::Markup {
    html! {
        p { b { "Contents for " (id) } }
        textarea #contents { (PreEscaped(contents)) }
        div #processing {}
        button #submit { "Submit" }
    }
}

pub fn user(user: UserInfo) -> maud::Markup {
    html! {
        p { b { "Name " (user.first_name) " " (user.last_name) } }
        p { "Privilege: " mono.info { (user.privilege) } }
        p { "Codes: " mono.info { (user.codes) } }
        p { "Widgets: " mono.info { (user.widgets) } }
        p { "History:" }
        ul {
            @for user in &user.history {
                li {
                    mono { (user.entry) }
                    " read "
                    utc { (user.timestamp) }
                }
            }
        }
        p { "Preferences:" }
        ul {
            @for setting in &user.preferences {
                li {
                    mono { (setting.setting) }
                    ": "
                    mono { (setting.switch) }
                }
            }
        }
        (edit_user(Some(&user)))
    }
}

pub fn edit_user(user: Option<&UserInfo>) -> maud::Markup {
    let (first_name, last_name) = match user {
        Some(UserInfo {
            first_name,
            last_name,
            ..
        }) => (first_name.as_ref(), last_name.as_ref()),
        None => ("", ""),
    };
    html! {
        label { "First name" }
        input #user-first-name maxlength="30" value=(first_name);
        label { "Last name" }
        input #user-last-name maxlength = "30" value=(last_name);
        button #submit { "Submit" }
    }
}

pub fn volume(volume: VolumeInfo) -> maud::Markup {
    html! {
        p { b { "Volume " mono { (volume.id) } } }
        p { "Volume count: " span.info { (volume.volume_count) } }
        p { "Content type: " mono.info { (volume.content_type) } }
        p { "Owner: " mono.info { (volume.owner) } }
        p { "Entries:" }
        ul {
            @for entry in &volume.entries {
                li {
                    mono { (entry.id) }
                    " — "
                    (entry.description)
                }
            }
        }
        (edit_volume(Some(&volume)))
    }
}

pub fn edit_volume(volume: Option<&VolumeInfo>) -> maud::Markup {
    let (title, subtitle) = match volume {
        Some(VolumeInfo {
            title, subtitle, ..
        }) => (title.as_ref(), subtitle.as_ref()),
        None => ("", ""),
    };
    html! {
        label { "Title" }
        input #volume-title maxlength="30" value=(PreEscaped(title));
        label { "Subtitle" }
        textarea #volume-subtitle maxlength = "150" { (PreEscaped(subtitle)) }
        button #submit { "Submit" }
    }
}

pub fn volumes(volumes: Volumes) -> maud::Markup {
    html! {
        p { "Volumes:" }
        ul {
            @for volume in volumes.0 {
                li {
                    mono { (PreEscaped(volume.0)) }
                    " — "
                    (PreEscaped(volume.1))
                }
            }
        }
    }
}

pub fn entry(entry: EntryInfo) -> maud::Markup {
    html! {
        p { b { "Entry " mono { (entry.id) } } }
        p {
            "Parent volume: "
            span.info { mono { (entry.parent_volume.0) } " " (entry.parent_volume.1) }
        }
        p { "Author: " mono.info { (entry.author) } }
        p { "Sections:" }
        ul {
            @for section in &entry.sections {
                li {
                    mono { (section.id) }
                    " — "
                    (PreEscaped(&section.description))
                }
            }
        }
        (edit_entry(Some(&entry)))
    }
}

pub fn edit_entry(entry: Option<&EntryInfo>) -> maud::Markup {
    let (title, description, summary) = match entry {
        Some(EntryInfo {
            title,
            description,
            summary,
            ..
        }) => (title.as_ref(), description.as_ref(), summary.as_ref()),
        None => ("", "", ""),
    };
    html! {
        label { "Title" }
        input #entry-title maxlength="30" value=(PreEscaped(title));
        label { "Description" }
        textarea #entry-description maxlength = "75" { (PreEscaped(description)) }
        label { "Summary" }
        textarea #entry-summary maxlength = "150" { (PreEscaped(summary)) }
        button #submit { "Submit" }
    }
}

pub fn section(section: SectionInfo) -> maud::Markup {
    html! {
        p { b { "Section " mono { (section.id) } } }
        p { "Parent entry: " mono.info { (section.parent_entry) } }
        p {
            "In entry: " span.info {
                (section.in_entry.0 + 1)
                "/"
                (section.in_entry.1)
            }
        }
        p { "Status: " mono.info { (section.status) } }
        p { "Length: " span.info { (section.length) } }
        (edit_section(Some(&section), &section.date))
        p { "Perspectives: " mono.info { (section.perspectives) }}
        p { "Comments: " }
        ul {
            @for comment in &section.comments {
                li {
                   mono { (comment.author) }
                   " on "
                   utc { (comment.timestamp) }
                   " — "
                   (PreEscaped(&comment.contents))
                }
            }
        }
    }
}

pub fn edit_section(section: Option<&SectionInfo>, date: &str) -> maud::Markup {
    let (heading, description, summary) = match section {
        Some(SectionInfo {
            heading,
            description,
            summary,
            ..
        }) => (heading.as_ref(), description.as_ref(), summary.as_ref()),
        None => ("", "", ""),
    };
    html! {
        label { "Heading" }
        input #section-heading maxlength="30" value=(PreEscaped(heading));
        label { "Description" }
        textarea #section-description maxlength="75" { (PreEscaped(description)) }
        label { "Summary" }
        textarea #section-summary maxlength="150" { (PreEscaped(summary)) }
        label { "Added" }
        input #section-date value=(date);
        button #submit { "Submit" }
    }
}
