use std::collections::HashSet;

use chrono::NaiveDate;

use super::*;
use crate::data::volume::Kind as VolumeKind;

pub enum EntryDestination {
    Top,
    Section(u32),
    Line(u32, usize),
}

pub fn entry(
    headers: &HeaderMap,
    entry: &Entry,
    user: &User,
    destination: EntryDestination,
) -> Markup {
    if entry.parent_volume().kind() == VolumeKind::Creative {
        return fiction_entry(headers, entry);
    }

    let mut jump_found = false;
    let mut section_html = |section: &Section, complete: bool| {
        let content = section.content();
        let mut is_commentary = 0;

        let content = content.lines().enumerate().map(|(line_index, line_text)| {
            if line_text.starts_with("/note") {
                let note_desc = line_text.split("/note ").nth(1);
                return PreEscaped(if let Some(note_desc) = note_desc {
                    format!(r#"<div class="note"><p class="note-desc">{note_desc}</p>"#)
                } else {
                    format!(r#"<div class="note">"#)
                });
            }
            if line_text.starts_with("/retro") {
                let note_timestamp = line_text.split("/retro ").nth(1);
                let date =
                    note_timestamp.and_then(|n| NaiveDate::parse_from_str(n, "%Y-%m-%d").ok());
                return if let Some(date) = date {
                    html! {
                        (PreEscaped(r#"<div class="retro">"#))
                        h3 { "Retrospective" }
                        p.retro-timestamp { "Added " (crate::data::date_string(&date)) }
                    }
                } else {
                    html! {
                        (PreEscaped(r#"<div class="retro">"#))
                        h3 { "Retrospective" }
                    }
                };
            }
            if line_text.starts_with("/comms") {
                let note_timestamp = line_text.split("/comms ").nth(1);
                let date =
                    note_timestamp.and_then(|n| NaiveDate::parse_from_str(n, "%Y-%m-%d").ok());
                is_commentary = 1;
                return if let Some(date) = date {
                    let date = crate::data::date_string(&date);
                    PreEscaped(format!(
                        r#"<div class="comm"><p class="comm-date">Commentary {date}</p><ol>"#,
                    ))
                } else {
                    PreEscaped(r#"<div class="comm"><p class="comm-date">Commentary</p><ol>"#.to_string())
                };
            }
            if line_text.starts_with("/comm") {
                is_commentary = 2;
                return PreEscaped(r#"<li>"#.to_string());
            }
            if line_text.starts_with("/end") {
                return if is_commentary == 1 {
                    is_commentary = 0;
                    PreEscaped("</ol></div>".to_string())
                } else if is_commentary == 2 {
                    is_commentary = 1;
                    PreEscaped("</li>".to_string())
                } else {
                    PreEscaped("</div>".to_string())
                };
            }
            if line_text.starts_with("/aside") {
                return PreEscaped(r#"<div class="aside">"#.to_string());
            }
            if line_text.starts_with("/img") {
                let mut parts = line_text.split(" ").skip(1);
                let url = parts.next().unwrap_or("");
                let caption = parts.collect::<Vec<_>>().join(" ");
                return html! {
                    .img {
                        img src={ "/image/" (url) ".jpg" };
                        p.caption { (PreEscaped(caption)) }
                        .open {
                            "Expand image"
                        }
                    }
                };
            }

            let (jump_here, jump_section) = match destination {
                EntryDestination::Section(s) if !jump_found && s == section.id() => {
                    jump_found = true;
                    (true, true)
                }
                EntryDestination::Line(s, line)
                    if !jump_found && s == section.id() && line_index >= line =>
                {
                    jump_found = true;
                    (true, false)
                }
                _ => (false, false),
            };

            let thread = section.comments(line_index);
            let commenters: HashSet<_> = thread.comments.iter().map(|c| &c.author).collect();

            html! {
                p.textline edat_line=(line_index) .here[jump_here] .here-section[jump_section] {
                    (PreEscaped(line_text))
                    @if commenters.len() > 0 {
                        span.open-comments { " ●" }
                    }
                }
            }
        });

        let show_timestamp = entry.parent_volume().kind() == VolumeKind::Journal;

        html! {
            .section edat_section=(section.id()) {
                @if let Some(heading) = section.heading() {
                    h3 { (PreEscaped(heading)) }
                }
                .body {
                    @if show_timestamp {
                        p.timestamp { "Added " (crate::data::date_string(&section.date())) }
                    }
                    .lines {
                        @for line in content {
                            (line)
                        }
                    }
                    @if !complete {
                        .aside {
                            p { b { "Incomplete section" } }
                        }
                    }
                }
            }
        }
    };

    let volume = if entry.parent_volume().parts_count() == 1 {
        html! {
            (PreEscaped(entry.parent_volume().title()))
        }
    } else {
        html! {
            (PreEscaped(entry.parent_volume().title()))
            " vol. "
            (roman::to(1 + entry.parent_volume_part() as i32).unwrap())
        }
    };

    let body = html! {
        h2.page-title { (PreEscaped(entry.title())) }
        a.volume href={ "/volume/" (entry.parent_volume_id()) } {
            (volume)
        }
        @if entry.section_count() == 0 {
            .section {
                .body {
                    .aside {
                        b { "Entry coming soon" }
                    }
                }
            }
        } @else {
            @for section in entry.sections() {
                @match section.status() {
                    section::Status::Complete => {
                        (section_html(&section, true))
                    }
                    section::Status::Incomplete => {
                        (section_html(&section, false))
                    }
                    section::Status::Missing => {
                        .section {
                            .body {
                                .aside {
                                    p.coming-soon { b { "Section coming soon" } }
                                    p { (section.description()) }
                                }
                            }
                        }
                    }
                }
                @if 1 + section.index_in_parent() != section.parent_entry().section_count() {
                    .divider {
                        .line {}
                    }
                }
            }
        }
    };

    let drawers = Vec::new();

    let topdrawer = html! {
        p.drawer-close { "✕" }
        nav #topnav2 {
            a href="/" { "HOME" }
            a href="/library" { "LIBRARY" }
            a href="/history" { "HISTORY" }
            a href="/music" { "MUSIC" }
            a href="/profile" { "PROFILE" }
        }
        #sectionnav {
            @for section in entry.sections() {
                @match section.status() {
                    section::Status::Complete | section::Status::Incomplete => {
                        @let unread = user.section_progress(&section).is_none();
                        @if let Some(heading) = section.heading() {
                            h3 { (PreEscaped(heading)) }
                        }
                        .topsection edat_section=(section.id()) .unread[unread]{
                            p.summary { (PreEscaped(section.summary())) }
                            p.status {
                                span.date { "Added " (crate::data::date_string(&section.date())) }
                                span.unread {
                                    @if unread {
                                        "Unread"
                                    }
                                }
                            }
                        }
                    }
                    section::Status::Missing => {
                        .topsection.missing {
                            p.summary { (PreEscaped(section.description())) }
                            p.status { "Coming soon" }
                        }
                    }
                }
            }
        }
    };

    let body = wrappers::standard(body, drawers, Some(topdrawer));

    wrappers::universal(body, headers, "entry", entry.title(), true)
}

pub fn fiction_entry(headers: &HeaderMap, entry: &Entry) -> Markup {
    let section_html = |section: &Section| {
        let content = section.content();
        let content = content.lines().enumerate().map(|(line_index, line_text)| {
            if line_text.starts_with("/note") {
                let note_desc = line_text.split("/note ").nth(1);
                return PreEscaped(if let Some(note_desc) = note_desc {
                    format!(r#"<div class="note"><p class="note-desc">{note_desc}</p>"#)
                } else {
                    format!(r#"<div class="note">"#)
                });
            }
            if line_text.starts_with("/end") {
                return PreEscaped("</div>".to_string());
            }

            html! {
                p.textline edat_line=(line_index) {
                    (PreEscaped(line_text))
                }
            }
        });

        html! {
            .section edat_section=(section.id()) {
                @if let Some(heading) = section.heading() {
                    h3 { (PreEscaped(heading)) }
                }
                .body {
                    .lines {
                        @for line in content {
                            (line)
                        }
                    }
                }
            }
        }
    };

    let body = html! {
        #titlecard {
            a.edat href="/" { "Every Day’s a Thursday" }
            h2.page-title { (PreEscaped(entry.title())) }
        }
        @for section in entry.sections() {
            (section_html(&section))
            @if 1 + section.index_in_parent() != section.parent_entry().section_count() {
                .divider {
                    .line {}
                }
            }
        }
    };

    wrappers::universal(body, headers, "fiction_entry", entry.title(), true)
}

pub fn error(headers: &HeaderMap, id: &str) -> Markup {
    let body = html! {
        h1 { "Every Day’s a Thursday" }
        p { "An entry with the id " mono { (id) } " does not appear to exist." }
    };

    wrappers::universal(body, headers, "missing_id", "Content not found", false)
}

pub fn section_error(headers: &HeaderMap, id: u32) -> Markup {
    let body = html! {
        h1 { "Every Day’s a Thursday" }
        p { "A section with the id " mono { (id) } " does not appear to exist." }
    };

    wrappers::universal(body, headers, "missing_id", "Content not found", false)
}
