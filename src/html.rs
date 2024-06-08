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
        html lang="en-us" {
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
}

pub struct WidgetOption {
    pub name: String,
    pub description: String,
    pub order: Option<usize>,
    pub id: String,
}

pub fn widget_options(widgets: Vec<WidgetOption>) -> Markup {
    html! {
        @for widget in widgets {
            .widget {
                @if let Some(order) = widget.order {
                    span style="opacity: 1" { "#" (order + 1) }
                } @else {
                    span {}
                }
                button #(widget.id) .selected[widget.order.is_some()] {
                    h3 { (PreEscaped(&widget.name)) }
                    p { (PreEscaped(&widget.description)) }
                }
            }
        }
    }
}

pub fn profile(headers: &HeaderMap, data: profile::ProfileData) -> Markup {
    let widgets = widgets(&data.widgets);

    let history_preview_length = 3;
    let history_preview = data.sections.iter().take(history_preview_length);
    let history_rest = data.sections.iter().skip(history_preview_length);

    let profile = html! {
        h1 { a href="/" { "Every Day’s a Thursday" } }
        #homepage.module {
            h2 { "Homepage settings"}
            .wrapper {
                p { "Choose which widgets to include on your homepage. Changes are saved automatically. The order you select them in will determine the order they appear on your homepage." }
                #widgets {
                    (widget_options(widgets))
                }
            }
            p.expand #homepage-expand { "Show options" }
        }
        #history.module {
            h2 { "Reading history" }
            .wrapper {
                p { "Only the last two months are shown. Select a section below to return to your place." }
                #history-preview {
                    @for section in history_preview {
                        (section.to_html())
                    }
                }
                #history-rest {
                    @for section in history_rest {
                        (section.to_html())
                    }
                }
            }
            p.expand #history-expand { "Show more" }
        }
        #contributions.module {
            h2 { "Contributions" }
            p { "The ability to write featured content is coming soon, including the Perspectives feature
                and Personal Journals. Your hub for writing and editing will be right here." }
        }
        button #home { "Go back" }
    };

    universal(profile, headers, "profile", "Profile")
}

pub mod profile {
    use maud::{html, Markup, PreEscaped};

    pub struct ProfileData {
        pub widgets: Vec<String>,
        pub sections: Vec<ViewedSection>,
    }

    pub struct ViewedSection {
        pub id: u32,
        pub description: String,
        pub timestamp: i64,
        pub entry: String,
        pub index: (usize, usize),
        pub progress: (usize, usize),
    }

    impl ViewedSection {
        pub fn to_html(&self) -> Markup {
            let progress = (self.progress.0 as f32 / self.progress.1 as f32 * 100.0).round();

            html! {
                a.section href={ "/section/" (self.id) "?line=" (self.progress.0) } {
                    h3 { (PreEscaped(&self.entry)) }
                    p.description {
                        (self.description)
                        span.index { (self.index.0 + 1) "/" (self.index.1) }
                    }
                    p.info { span.progress { (progress) "% complete" } span.lastread { "Last read " utc { (self.timestamp) } } }
                }
            }
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

pub fn widgets(selected: &[String]) -> Vec<WidgetOption> {
    use WidgetOption as W;

    let order = |id| selected.iter().position(|s| s == id);

    vec![
        W {
            name: "Recent additions".to_owned(),
            description: "Carousel of the latest sections".to_owned(),
            order: order(&"recent-widget"),
            id: "recent-widget".to_owned(),
        },
        W {
            name: "The library".to_owned(),
            description: "Quick access to the main journal’s four books".to_owned(),
            order: order(&"library-widget"),
            id: "library-widget".to_owned(),
        },
        W {
            name: "Last read".to_owned(),
            description: "Return to where you left off".to_owned(),
            order: order(&"last-widget"),
            id: "last-widget".to_owned(),
        },
        W {
            name: "Conversations".to_owned(),
            description: "See where readers have recently commented".to_owned(),
            order: order(&"conversations-widget"),
            id: "conversations-widget".to_owned(),
        },
        W {
            name: "Random entry".to_owned(),
            description: "Reading recommendation".to_owned(),
            order: order(&"random-widget"),
            id: "random-widget".to_owned(),
        },
        W {
            name: "Extras".to_owned(),
            description: "Quick access to old journals, fiction, and more".to_owned(),
            order: order(&"extras-widget"),
            id: "extras-widget".to_owned(),
        },
        W {
            name: "Search bar".to_owned(),
            description: "Website search features".to_owned(),
            order: order(&"search-widget"),
            id: "search-widget".to_owned(),
        },
    ]
}

pub fn setup(headers: &HeaderMap, volumes: Vec<setup::Volume>) -> Markup {
    let widgets = widgets(&[]);

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
            (widget_options(widgets))
            p { "You can always change these settings later." }
            button #done { "Finished" }
        }
    };
    universal(setup, headers, "setup", "Setup account")
}

pub mod home {
    use maud::{html, Markup, PreEscaped};

    use crate::data;

    pub struct LibraryWidget {
        pub volumes: Vec<LibraryVolume>,
    }

    impl Widget for LibraryWidget {
        fn html(&self) -> Markup {
            html! {
                h2 { "The library" }
                .volumes {
                    @for volume in &self.volumes {
                        a.volume href={ "/volume/" (volume.id) } {
                            h3 { (PreEscaped(&volume.title)) }
                            @if let Some(subtitle) = &volume.subtitle {
                                p.subtitle { (PreEscaped(subtitle)) }
                            }
                        }
                    }
                }
            }
        }

        fn id(&self) -> &'static str {
            "library-widget"
        }
    }

    pub struct LibraryVolume {
        pub title: String,
        pub id: String,
        pub subtitle: Option<String>,
        pub entry_count: usize,
    }

    pub struct RecentWidget {
        pub sections: Vec<RecentSection>,
        pub expand: bool,
    }

    impl RecentWidget {
        fn section(&self, section: &RecentSection, show_previous: bool) -> Markup {
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
                .section edat-unread[section.read.is_none()] {
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
                    @if let Some(ref time) = section.read {
                        span.read { "You read on " utc { (time) } }
                    } @else {
                        span.unread-wrapper {
                            span.unread { "Unread" }
                            button.skip edat-section=(section.id) { "I’ve already read this" }
                        }
                    }
                    @if let (true, Some((id, description))) = (show_previous, &section.previous) {
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
                        @let show_previous = section
                            .previous
                            .as_ref()
                            .is_some_and(|p| !self.sections.iter().any(|s| s.id != p.0));
                        (self.section(section, show_previous))
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
        pub read: Option<i64>,
    }

    pub trait Widget {
        fn html(&self) -> Markup;

        fn id(&self) -> &'static str;
    }
}

pub fn home(
    headers: &HeaderMap,
    widgets: Vec<Box<dyn home::Widget>>,
    introduction: Vec<&str>,
) -> maud::Markup {
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
                    a href="/profile" { "Go to settings" }
                }
            }
            .widget #intro-widget {
                h2 { "Introduction" }
                .introduction {
                    @for line in &introduction {
                        p { (PreEscaped(line)) }
                    }
                }
            }
        }
    };
    universal(body, &headers, "home", "Home")
}

pub fn terminal(headers: &HeaderMap, allowed: bool) -> maud::Markup {
    let body = if allowed {
        html! {
            h1 { b { "Command terminal" } }
            input #command type="text"
                placeholder="Enter command here"
                autocomplete="off"
                autocapitalize="off"
                spellcheck="false"
                autofocus {}
            p #invalid-command style="opacity: 0.0" { "Invalid command" }
            #response {

            }
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

    use maud::{html, PreEscaped};

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
        pub date: i64,
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

    pub fn error(category: &str, id: impl Display) -> maud::Markup {
        html! {
            p.error { "Unknown " (category) " " mono { (id) } }
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

    pub fn contents(id: impl Display, contents: String) -> maud::Markup {
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
                        utc { (user.date) }
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
}
