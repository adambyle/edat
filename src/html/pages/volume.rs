use chrono::NaiveDate;
use crate::data::volume::Kind as VolumeKind;

use super::*;

pub fn volume(headers: &HeaderMap, volume: &Volume, user: &User) -> Markup {
    let introduction = volume.intro();
    let show_intro = !introduction.is_empty();

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

            if volume.kind() != VolumeKind::Journal {
                break 'status html!{};
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

            if entry
                .sections()
                .any(|s| s.status() != section::Status::Complete)
            {
                if let Some(last_edited) = last_edited {
                    break 'status html! {
                        span.incomplete {
                            "Last edited "
                            (date_string(&last_edited))
                        }
                    };
                }
            }

            html! {
                span.complete { "Complete" }
            }
        };

        let is_creative = volume.kind() == VolumeKind::Creative;

        html! {
            a.entry href={ "/entry/" (entry.id()) } {
                h4 .creative-title[is_creative] { (PreEscaped(entry.title())) }
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
        @if show_intro {
            #intro {
                h3 { "Introduction" }
                #intro-text {
                    @for line in introduction.lines() {
                        p { (PreEscaped(line)) }
                    }
                }
            }
        }
    };

    // Gather unfinished entry suggestions.
    let unfinished_entries: Vec<_> = volume
        .entries()
        .filter(|e| {
            e.sections()
                .any(|s| s.status() == section::Status::Complete)
            && e.parent_volume().kind() == VolumeKind::Journal
        })
        .filter_map(|e| {
            Some(match user.entry_progress(&e) {
                Some(EntryProgress::Finished { .. }) => return None,
                Some(EntryProgress::UpToSection {
                    section_id,
                    section_index,
                    ..
                }) => {
                    let section = user.index().section(section_id).unwrap();
                    let description = section.description();
                    html! {
                        .suggestion {
                            a.entry-link href={ "/section/" (section_id) } {
                                h4 { (PreEscaped(e.title())) }
                                p.position {
                                    "Start section "
                                    (section_index + 1)
                                }
                            }
                            p.description { (PreEscaped(description)) }
                            button.skip edat_section=(section_id) { "Mark as read" }
                        }
                    }
                }
                Some(EntryProgress::InSection {
                    section_id,
                    section_index,
                    line,
                    ..
                }) => {
                    let section = user.index().section(section_id).unwrap();
                    let description = section.description();
                    html! {
                        .suggestion {
                            a.entry-link href={ "/section/" (section_id) "?line=" (line) } {
                                h4 { (PreEscaped(e.title())) }
                                p.position {
                                    "Continue section "
                                    (section_index + 1)
                                }
                            }
                            p.description { (PreEscaped(description)) }
                            button.skip edat_section=(section_id) { "Mark as read" }
                        }
                    }
                }
                None => {
                    let first_section_id = e
                        .sections()
                        .find(|s| s.status() == section::Status::Complete)
                        .unwrap()
                        .id();
                    html! {
                        .suggestion {
                            a.entry-link href={ "/entry/" (e.id()) } {
                                h4 { (PreEscaped(e.title())) }
                                p.position { "Start this entry" }
                            }
                            button.skip edat_section=(first_section_id) { "Mark as read"}
                        }
                    }
                }
            })
        })
        .collect();

    let drawers = if !unfinished_entries.is_empty() {
        vec![html! {
            #unread-drawer {
                div {
                    p.drawer-close { "✕" }
                    p { "Select an entry name to jump in."}
                    .unread-entries {
                        @for entry in unfinished_entries {
                            (entry)
                        }
                    }
                }
            }
        }]
    } else {
        Vec::new()
    };

    let body = wrappers::standard(body, drawers, None);

    wrappers::universal(body, headers, "volume", volume.title(), false)
}

pub fn error(headers: &HeaderMap, id: &str) -> Markup {
    let body = html! {
        h1 { "Every Day’s a Thursday" }
        p { "A volume with the id " mono { (id) } " does not appear to exist." }
    };

    wrappers::universal(body, headers, "missing_id", "Content not found", false)
}

pub fn library(headers: &HeaderMap, index: &Index) -> Markup {
    let entry_html = |entry: &Entry| {
        // Show volume part below the first entry in the part
        // if the volume has multiple parts.
        let vol = (entry.index_in_parent_volume_part() == 0
            && entry.parent_volume().parts_count() > 1)
            .then_some(format!(
                "Volume {}",
                (roman::to(1 + entry.parent_volume_part() as i32).unwrap())
            ));

        let status = 'status: {
            if !entry
                .sections()
                .any(|s| s.status() != section::Status::Missing)
            {
                break 'status html! {
                    span.incomplete { "Coming soon" }
                };
            }

            if entry.parent_volume().kind() != VolumeKind::Journal {
                break 'status html!{};
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

            if entry
                .sections()
                .any(|s| s.status() != section::Status::Complete)
            {
                if let Some(last_edited) = last_edited {
                    break 'status html! {
                        span.incomplete {
                            "Last edited "
                            (date_string(&last_edited))
                        }
                    };
                }
            }

            html! {
                span.complete { "Complete" }
            }
        };

        let is_creative = entry.parent_volume().kind() == VolumeKind::Creative;

        html! {
            .entry-wrapper {
                a.entry href={ "/entry/" (entry.id()) } {
                    h4 .creative-title[is_creative] { (PreEscaped(entry.title())) }
                    p.description { (PreEscaped(entry.description())) }
                    p.info {
                        @if entry.length() > 0 {
                            span.words { (PreEscaped(entry.length_string())) " words" }
                        }
                        (status)
                    }
                }
                @if let Some(vol) = vol {
                    p.volume-part { (vol) }
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

    let drawers = vec![html! {
        #search-drawer {
            div {
                p.drawer-close { "✕" }
                .search-box {
                    p { "Search the library" }
                    input
                        id="search-input"
                        type="text"
                        maxlength="40"
                        placeholder="Search for a collection or entry";
                }
                .results {

                }
            }
        }
    }];

    let body = wrappers::standard(body, drawers, None);

    wrappers::universal(body, headers, "library", "The library", false)
}
