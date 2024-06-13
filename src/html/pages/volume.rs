use chrono::NaiveDate;

use super::*;

pub fn volume(headers: &HeaderMap, volume: &Volume, user: &User) -> Markup {
    let introduction = volume.intro();
    let entries = volume.entries_by_part();
    let show_volume_num = volume.parts_count() > 1;

    let entry_html = |entry: &Entry| {
        let status = 'status: {
            if !entry
                .sections()
                .any(|s| s.status() != section::Status::Missing)
            {
                break 'status html! {
                    span.incomplete { "Coming soon" }
                };
            }

            let mut last_edited: Option<NaiveDate> = None;
            for section in entry.sections() {
                if section.status() != section::Status::Missing {
                    last_edited = Some(match last_edited {
                        None => section.date(),
                        Some(date) => date.max(section.date()),
                    });
                }
            }
            if let Some(last_edited) = last_edited {
                break 'status html! {
                    span.incomplete {
                        "Last edited "
                        (date_string(&last_edited))
                    }
                };
            }

            html! {
                span.complete { "Complete" }
            }
        };

        html! {
            a.entry href={ "/entry/" (entry.id()) } {
                h4 { (PreEscaped(entry.title())) }
                p.summary { (PreEscaped(entry.summary())) }
                p.info {
                    @if entry.length() > 0 {
                        span.words { (PreEscaped(entry.length_string())) " words" }
                    }
                    (status)
                }
            }
        }
    };

    let body = html! {
        h2.page-title { (PreEscaped(volume.title())) }
        @if let Some(subtitle) = volume.subtitle() {
            p.subtitle { (PreEscaped(subtitle)) }
        }
        @for (part, entries) in entries {
            .part {
                @if show_volume_num {
                    h3 {
                        "Volume "
                        (roman::to(1 + part as i32).unwrap())
                    }
                }
                .entries {
                    @for entry in entries {
                        (entry_html(&entry))
                    }
                }
            }
        }
        #intro {
            h3 { "Introduction" }
            #intro-text {
                @for line in introduction.lines() {
                    p { (PreEscaped(line)) }
                }
            }
        }
    };

    let body = wrappers::standard(body);

    wrappers::universal(body, headers, "volume", volume.title())
}

pub fn error(headers: &HeaderMap, id: &str) -> Markup {
    let body = html! {
        h1 { "Every Day’s a Thursday" }
        p { "A volume with the id " mono { (id) } " does not appear to exist." }
    };

    wrappers::universal(body, headers, "missing_id", "Content not found")
}

pub fn library(headers: &HeaderMap, index: &Index) -> Markup {
    let entry_html = |entry: &Entry| {
        let status = 'status: {
            if !entry
                .sections()
                .any(|s| s.status() != section::Status::Missing)
            {
                break 'status html! {
                    span.incomplete { "Coming soon" }
                };
            }

            let mut last_edited: Option<NaiveDate> = None;
            for section in entry.sections() {
                if section.status() != section::Status::Missing {
                    last_edited = Some(match last_edited {
                        None => section.date(),
                        Some(date) => date.max(section.date()),
                    });
                }
            }
            if let Some(last_edited) = last_edited {
                break 'status html! {
                    span.incomplete {
                        "Last edited "
                        (date_string(&last_edited))
                    }
                };
            }

            html! {
                span.complete { "Complete" }
            }
        };

        html! {
            a.entry href={ "/entry/" (entry.id()) } {
                h4 { (PreEscaped(entry.title())) }
                p.description { (PreEscaped(entry.description())) }
                p.info {
                    @if entry.length() > 0 {
                        span.words { (PreEscaped(entry.length_string())) " words" }
                    }
                    (status)
                }
            }
        }
    };
    
    let volume_html = |volume: &Volume| {
        html! {
            .volume {
                a.title href={ "/volume/" (volume.id()) } {
                    h3 { (PreEscaped(volume.title())) }
                    @if let Some(subtitle) = volume.subtitle() {
                        p.subtitle { (PreEscaped(subtitle)) }
                    }
                }
                .entries {
                    @for entry in volume.entries() {
                        (entry_html(&entry))
                    }
                }
            }
        }
    };

    let body = html! {
        h2 { "The library" }
        #volumes {
            @for volume in index.volumes() {
                (volume_html(&volume))
            }
        }
    };

    let body = wrappers::standard(body);

    wrappers::universal(body, headers, "library", "The library")
}
