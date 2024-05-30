use axum::http::HeaderMap;
use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::routes;

fn universal(
    body: PreEscaped<String>,
    headers: &HeaderMap,
    resource: &'static str,
    title: &str,
) -> Markup {
    let dark_theme = match routes::get_cookie(headers, "edat_theme") {
        Some("dark") => Some("dark-theme"),
        _ => None,
    };

    html! {
        (DOCTYPE)
        head {
            title { "Every Day’s a Thursday | " (title) }
            meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no";
            link type="text/css" rel="stylesheet" href={"style/" (resource) ".css"};
        }
        body class=[dark_theme] {
            (body)
            script type="module" src={"script/" (resource) ".js"} {};
        }
    }
}

pub fn login(headers: &HeaderMap) -> Markup {
    let login = html! {
        h1 { "Every Day’s a Thursday" }
        p { b { "Please enter your credentials below." } }
        p { "You should only need to do this once per device if cookies are enabled. Enter your name below (first or full) and your access code. Letter case does not matter." }
        ul {
            li {
                label for="name" { "NAME" }
                input #name-input name="name" type="text";
            }
            li {
                label for="code" { "ACCESS CODE" }
                input #code-input name="code" type="text";
            }
            li {
                button type="submit" id="login-button" { "LOGIN" }
            }
        }
        p #error-msg style="display: none;" { "Invalid credentials." }
    };
    universal(login, headers, "login", "Login")
}

pub mod setup {
    pub struct Entry {
        pub id: String,
        pub title: String,
        pub description: String,
    }

    pub struct Volume {
        pub title: String,
        pub entries: Vec<Entry>,
    }
}

pub fn setup(headers: &HeaderMap, volumes: Vec<setup::Volume>) -> Markup {
    let setup = html! {
        #welcome {
            h1 { "Every Day’s a Thursday" }
            p { "To improve your experience, the website makes recommendations based on your reading log." }
            p { "If the website ever recommends you something you have already read, please be willing to select \"" b {"I have already read this"} "\" to improve the log." }
            p { "Right now, your log does not exist. What would you like the recommendation system to know?" }
            ul {
                li { "If the system assumes you have read nothing, it will recommend you all the entries, including ones you have read before." }
                li { "If the system assumes you have read everything, it will only recommend you new releases after this point." }
                li { b { "Recommended: " } "You may also specify which entries you have already read so that the system can make precise recommendations. This process will take no longer than two minutes." }
            }
            p { b { "How would you like to begin?" } }
            button.record-choice #blank-record { "Assume I have read nothing" }
            button.record-choice #full-record { "Assume I have read everything" }
            button.record-choice #custom-record { "I will specify which entries I have read" }
        }
        #choose-entries {
            p { "Using the best of your knowledge, select the entries below that you believe you may have read before." }
            @for volume in volumes {
                h2.volume { (PreEscaped(volume.title)) }
                @for entry in volume.entries {
                    .entry edat-entry=(entry.id) {
                        h3 { (PreEscaped(entry.title)) }
                        p { (PreEscaped(entry.description)) }
                    }
                }
            }
        }
        #configure {
            p { b { "Your homepage is customizable to serve the most relevant content." } }
            p { "Select the elements below in the order (top to bottom) you would like them to appear on your homepage. You can include or omit whichever you want." }
            p { "Common resources, like the library, the index, and the addition history, will always have quick links at the top, but you can get more detailed information by selecting their widgets below." }
            .widget {
                span {}
                button #recent-widget {
                    h3 { "Recent additions" }
                    p { "Carousel of the latest sections" }
                }
            }
            .widget {
                span {}
                button #library-widget {
                    h3 { "The library" }
                    p { "Quick access to the main journal’s four books" }
                }
            }
            .widget {
                span {}
                button #conversations-widget {
                    h3 { "Conversations" }
                    p { "See where readers have recently commented" }
                }
            }
            .widget {
                span {}
                button #random-widget {
                    h3 { "Random entry" }
                    p { "Reading recommendation" }
                }
            }
            .widget {
                span {}
                button #extras-widget {
                    h3 { "Extras" }
                    p { "Quick access to old journals, fiction, and more" }
                }
            }
            .widget {
                span {}
                button #search-widget {
                    h3 { "Search bar" }
                    p { "Website search feature" }
                }
            }
            p { "You can always change these settings later." }
            button #done { "Finished" }

        }
    };
    universal(setup, headers, "setup", "Setup account")
}

pub mod home {
    use maud::{html, Markup, PreEscaped};

    use crate::data;

    pub struct RecentWidget {
        pub sections: Vec<RecentSection>,
        pub expand: bool,
    }

    impl RecentWidget {
        fn section(&self, section: &RecentSection) -> Markup {
            html! {
                @let concise = html! {
                    p.description { (PreEscaped(&section.description)) }
                };
                @let detailed = html! {
                    p.summary { (PreEscaped(&section.summary)) }
                    span.details {
                        span.index {
                            @if section.in_entry.1 == 1 {
                                "Standalone"
                            } @else {
                                "Section " (section.in_entry.0)
                            }
                        }
                        span.wordcount {
                            (section.length) " words"
                        }
                        span.date {
                            "Added " (section.date)
                        }
                    }
                };
                .section {
                    a.section-info href={ "/section/" (section.id) } {
                        @let volume = html! {
                            @if section.parent_volume.2 == 1 {
                                (PreEscaped(&section.parent_volume.0))
                            } @else {
                                (PreEscaped(&section.parent_volume.0))
                                " vol. "
                                (data::roman_numeral(section.parent_volume.1 + 1))
                            }
                        };
                        @if self.expand {
                            p.volume.detailed {
                                (volume)
                            }
                        } @else {
                            p.volume.detailed
                                style="display: none"
                            {
                                (volume)
                            }
                        }
                        h3 { (PreEscaped(&section.parent_entry)) }
                        @if self.expand {
                            .concise style="display: none" {
                                (concise)
                            }
                            .detailed {
                                (detailed)
                            }
                        } @else {
                            .concise {
                                (concise)
                            }
                            .detailed style="display: none" {
                                (detailed)
                            }
                        }
                    }
                    @if !section.read {
                        span.unread { "UNREAD" }
                    } @else {
                        span.unread style="opacity: 0" { "UNREAD" }
                    }
                    @if let Some((id, ref description)) = section.previous {
                        @let previous = html! {
                            p.previous-label {"Previous section"}
                            p.previous-description { (PreEscaped(description)) }
                        };
                        @if self.expand {
                            a.previous.detailed
                                href={"/section/" (id)}
                            {
                                (previous)
                            }
                        } @else {
                            a.previous.detailed
                                style="display: none"
                                href={"/section/" (id)}
                            {
                                (previous)
                            }
                        }
                    }
                }
            }
        }
    }

    impl Widget for RecentWidget {
        fn html(&self) -> Markup {
            let detail_class = if self.expand {
                "show-detailed"
            } else {
                "show-concise"
            };
            html! {
                h2 { "Recent uploads" }
                #recent-carousel class=(detail_class) {
                    @for section in &self.sections {
                        (self.section(section))
                    }
                }
                button id="recent-expand" {
                    @if self.expand {
                        span.detailed { "Hide details" }
                        span.concise style="display: none" { "Show details" }
                    } @else {
                        span.detailed style="display: none" { "Hide details" }
                        span.concise { "Show details" }
                    }
                }
            }
        }

        fn id(&self) -> &'static str {
            "recent-widget"
        }
    }

    pub struct RecentSection {
        pub id: u32,
        pub parent_entry: String,
        pub parent_volume: (String, usize, usize),
        pub in_entry: (usize, usize),
        pub previous: Option<(u32, String)>,
        pub description: String,
        pub summary: String,
        pub date: String,
        pub length: String,
        pub read: bool,
    }

    pub trait Widget {
        fn html(&self) -> Markup;

        fn id(&self) -> &'static str;
    }
}

pub fn home(headers: &HeaderMap, widgets: Vec<Box<dyn home::Widget>>) -> maud::Markup {
    let body = html! {
        h1 #title { span { "Every Day’s a Thursday" } }
        main {
            nav #topnav {
                a href="/history" { "HISTORY" }
                a href="/library" { "LIBRARY" }
                a href="/index" { "INDEX" }
                a href="/profile" { "PROFILE" }
            }
            @for widget in &widgets {
                .widget #(widget.id()) {
                    (widget.html())
                }
            }
            @if widgets.len() == 0 {
                .widget #empty-widget {
                    h2 { "Customize your homepage" }
                    p { "You haven’t added any elements to your homepage yet, like quick access to recent entries or library shortcuts, but you can do so in your settings." }
                    a href="/me" { button { "Go to settings" } }
                }
            }
        }
        p { "TEST" }
    };
    universal(body, &headers, "home", "Home")
}

pub fn terminal(headers: &HeaderMap, allowed: bool) -> maud::Markup {
    let body = if allowed {
        html! {
            p { b { "Terminal" } }
            input #command type="text"
                placeholder="Enter command here"
                autocomplete="off"
                autocapitalize="off"
                spellcheck="false"
                autofocus {}
        }
    } else {
        html! {
            p #forbidden { "You do not have access to the terminal" }
        }
    };
    universal(body, headers, "terminal", "Terminal")
}

pub mod terminal {
    use std::fmt::Display;

    use maud::html;

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
        pub date: String,
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
        pub timestamp: String,
        pub contents: String,
    }

    pub struct Volumes(pub Vec<(String, String)>);

    pub fn error(category: &str, id: impl Display) -> maud::Markup {
        html! {
            p.error { "Unknown " (category) " " (id) }
        }
    }

    pub fn user(user: UserInfo) -> maud::Markup {
        html! {
            p { b { "Name " (user.first_name) " " (user.last_name) } }
            p { "Privilege: " span.info { (user.privilege) } }
            p { "Codes: " span.info { (user.codes) } }
            p { "Widgets: " span.info { (user.widgets) } }
            p { "History:" }
            ul {
                @for user in &user.history {
                    li {
                        (user.entry)
                        " read "
                        (user.date)
                    }
                }
            }
            p { "Preferences:" }
            ul {
                @for setting in &user.preferences {
                    li {
                        (setting.setting)
                        ": "
                        (setting.switch)
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
            input #user-first-name maxlength="30" { (first_name) }
            label { "Last name" }
            textarea #user-last-name maxlength = "150" { (last_name) }
            button #submit { "Submit" }
        }
    }

    pub fn volume(volume: VolumeInfo) -> maud::Markup {
        html! {
            p { b { "Volume " (volume.id) } }
            p { "Volume count: " span.info { (volume.volume_count) } }
            p { "Content type: " span.info { (volume.content_type) } }
            p { "Owner: " span.info { (volume.owner) } }
            p { "Entries:" }
            ul {
                @for entry in &volume.entries {
                    li {
                        (entry.id)
                        " - "
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
                title,
                subtitle,
                ..
            }) => (title.as_ref(), subtitle.as_ref()),
            None => ("", ""),
        };
        html! {
            label { "Title" }
            input #volume-title maxlength="30" { (title) }
            label { "Subtitle" }
            textarea #volume-subtitle maxlength = "150" { (subtitle) }
            button #submit { "Submit" }
        }
    }

    pub fn volumes(volumes: Volumes) -> maud::Markup {
        html! {
            p { "Volumes:" }
            ul {
                @for volume in volumes.0 {
                    li {
                        (volume.0)
                        " - "
                        (volume.1)
                    }
                }
            }
        }
    }

    pub fn entry(entry: EntryInfo) -> maud::Markup {
        html! {
            p { b { "Entry " (entry.id) } }
            p {
                "Parent volume: "
                span.info { (entry.parent_volume.0) (entry.parent_volume.1) }
            }
            p { "Author: " span.info { (entry.author) } }
            p { "Sections:" }
            ul {
                @for section in &entry.sections {
                    li {
                        (section.id)
                        " - "
                        (section.description)
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
            input #entry-title maxlength="30" { (title) }
            label { "Description" }
            textarea #entry-description maxlength = "75" { (description) }
            label { "Summary" }
            textarea #entry-summary maxlength = "150" { (summary) }
            button #submit { "Submit" }
        }
    }

    pub fn section(section: SectionInfo) -> maud::Markup {
        html! {
            p { b { "Section " (section.id) } }
            p { "Parent entry: " span.info { (section.parent_entry) } }
            p {
                "In entry: " span.info {
                    (section.in_entry.0 + 1)
                    "/"
                    (section.in_entry.1)
                }
            }
            p { "Status: " span.info { (section.status) } }
            p { "Added: " span.info { (section.date) } }
            p { "Length: " span.info { (section.length) } }
            (edit_section(Some(&section)))
            p { "Perspectives: " span.info { (section.perspectives) }}
            p { "Comments: " }
            ul {
                @for comment in &section.comments {
                    li {
                       (comment.author)
                       " on "
                       (comment.timestamp)
                       " - "
                       (comment.contents)
                    }
                }
            }
        }
    }

    pub fn edit_section(section: Option<&SectionInfo>) -> maud::Markup {
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
            input #section-heading maxlength="30" { (heading) }
            label { "Description" }
            textarea #section-description maxlength = "75" { (description) }
            label { "Summary" }
            textarea #section-summary maxlength = "150" { (summary) }
            button #submit { "Submit" }
        }
    }
}
