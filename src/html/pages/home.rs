use super::*;

pub fn home(
    headers: &HeaderMap,
    widgets: Vec<Box<dyn home::Widget>>,
    introduction: Vec<&str>,
) -> maud::Markup {
    let body = html! {
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
    };
    let body = wrappers::standard(body);
    wrappers::universal(body, &headers, "home", "Home")
}

pub enum RandomWidget {
    Unstarted(RandomEntry),
    Unfinished {
        entry: RandomEntry,
        section_id: u32,
        section_index: usize,
        progress: usize,
    },
    ReadAgain {
        entry: RandomEntry,
        last_read: i64,
    },
}

impl Widget for RandomWidget {
    fn html(&self) -> Markup {
        let entry_html = |entry: &RandomEntry, url: String| {
            let volume = html! {
                @if let Some(part) = entry.volume_part {
                    (PreEscaped(&entry.volume))
                    " vol. "
                    (roman::to(part as i32 + 1).unwrap())
                } @else {
                    (PreEscaped(&entry.volume))
                }
            };
            html! {
                a.entry href=(url) {
                    p.volume { (volume) }
                    h3 { (PreEscaped(&entry.title)) }
                    p.summary { (PreEscaped(&entry.summary)) }
                }
            }
        };

        let entry_wrapper = match self {
            Self::Unstarted(entry) => {
                let url = format!("/entry/{}", &entry.id);
                html! {
                    (entry_html(entry, url))
                    p.label { "You haven’t started this entry" }
                }
            }
            Self::Unfinished {
                entry,
                section_id,
                section_index,
                progress,
            } => {
                let url = format!("/section/{}?line={}", section_id, progress);
                html! {
                    (entry_html(entry, url))
                    @if *progress == 0 {
                        p.label { "You need to start section " (section_index + 1) }
                    } @else {
                        p.label { "You’re partway through section " (section_index + 1) }
                    }
                }
            }
            Self::ReadAgain { entry, last_read } => {
                let url = format!("/entry/{}", &entry.id);
                html! {
                    (entry_html(entry, url))
                    p.label { "You haven’t read this since " utc { (last_read) } }
                }
            }
        };

        html! {
            h2 { "Reading recommendation" }
            (entry_wrapper)
        }
    }

    fn id(&self) -> &'static str {
        "random-widget"
    }
}

pub struct RandomEntry {
    pub id: String,
    pub volume: String,
    pub volume_part: Option<usize>,
    pub title: String,
    pub summary: String,
}

pub struct LastWidget {
    pub section: Option<LastSection>,
}

impl Widget for LastWidget {
    fn html(&self) -> Markup {
        html! {
            h2 { "Last read" }
            @if let Some(ref section) = self.section {
                @let progress =
                    (section.progress.0 as f32 / section.progress.1 as f32 * 100.0)
                    .round();
                a.see-profile href="/profile" {
                    "See reading history in your profile"
                }
                a.last-section href={ "/section/" (section.id) "?line=" (section.progress.0) } {
                    h3 { (PreEscaped(&section.entry)) }
                    p.summary {
                        (PreEscaped(&section.summary))
                        span.index { (section.index.0 + 1) "/" (section.index.1) }
                    }
                    p.info {
                        span.progress { (progress) "% complete" }
                        span.lastread { "Last read " utc { (section.timestamp) } }
                    }
                }
            } @else {
                .last-section {
                    p { "You have no unfinished reading to pick up on." }
                }
            }
        }
    }

    fn id(&self) -> &'static str {
        "last-widget"
    }
}

pub struct LastSection {
    pub id: u32,
    pub summary: String,
    pub timestamp: i64,
    pub entry: String,
    pub index: (usize, usize),
    pub progress: (usize, usize),
}

pub struct LibraryWidget {
    pub volumes: Vec<LibraryVolume>,
    pub title: String,
}

impl Widget for LibraryWidget {
    fn html(&self) -> Markup {
        html! {
            h2 { (self.title) }
            .volumes {
                @for volume in &self.volumes {
                    a.volume href={ "/volume/" (volume.id) } {
                        h3 { (PreEscaped(&volume.title)) }
                        @if let Some(subtitle) = &volume.subtitle {
                            p.subtitle { (PreEscaped(subtitle)) }
                        }
                    }
                }
                a.volume-link href="/library" { "Search the full library" }
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
                            (roman::to(section.parent_volume.1 as i32 + 1).unwrap())
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
