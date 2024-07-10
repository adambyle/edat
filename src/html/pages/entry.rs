use std::collections::HashSet;

use super::*;

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
    let mut jump_found = false;
    let mut section_html = |section: &Section, complete: bool| {
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
            let commenters: HashSet<_> = thread
                .comments
                .iter()
                .map(|c| &c.author)
                .collect();
            // let author_text = if commenters.len() == 0 {
            //     String::new()
            // } else if commenters.len() == 1 {
            //     format!("See comments from {}", commenters.iter().next().unwrap().full_name())
            // } else if commenters.len() == 2 {
            //     let mut commenters = commenters.iter();
            //     format!("See comments from {} and {}", commenters.next().unwrap().first_name(), commenters.next().unwrap().first_name())
            // } else {
            //     format!("See comments from {} and {} others", commenters.iter().next().unwrap().first_name(), commenters.len() - 1)
            // };

            html! {
                p.textline edat_line=(line_index) .here[jump_here] .here-section[jump_section] {
                    (PreEscaped(line_text))
                    @if commenters.len() > 0 {
                        span.open-comments { " ●" }
                    }
                }
            }
        });

        html! {
            .section edat_section=(section.id()) {
                @if let Some(heading) = section.heading() {
                    h3 { (PreEscaped(heading)) }
                }
                .body {
                    p.timestamp { "Added " (crate::data::date_string(&section.date())) }
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
            a href="/forum" { "FORUM" }
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
