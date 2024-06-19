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
        let content = content.lines().enumerate().map(|(i, line)| {
            if line.starts_with("/note") {
                let note_desc = line.split("/note ").nth(1);
                return PreEscaped(if let Some(note_desc) = note_desc {
                    format!(r#"<div class="note"><p class="note-desc">{note_desc}</p>"#)
                } else {
                    format!(r#"<div class="note">"#)
                });
            }
            if line.starts_with("/end") {
                return PreEscaped("</div>".to_string());
            }
            if line.starts_with("/aside") {
                return PreEscaped(r#"<div class="aside">"#.to_string());
            }
            if line.starts_with("/img") {
                let mut parts = line.split(" ").skip(1);
                let url = parts.next().unwrap();
                let caption = parts.next().unwrap();
                return html! {
                    .img {
                        img src={ (url) };
                        p.caption { (PreEscaped(caption)) }
                        .open {
                            "Expand image"
                        }
                    }
                };
            }

            let jump_here = match destination {
                EntryDestination::Section(s) if !jump_found && s == section.id() => {
                    jump_found = true;
                    true
                }
                EntryDestination::Line(s, line)
                    if !jump_found && s == section.id() && i >= line =>
                {
                    jump_found = true;
                    true
                }
                _ => false,
            };

            html! {
                p .textline edat_line=(i) .here[jump_here] { (PreEscaped(line)) }
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

    let body = wrappers::standard(body, drawers);

    wrappers::universal(body, headers, "entry", entry.title())
}

pub fn error(headers: &HeaderMap, id: &str) -> Markup {
    let body = html! {
        h1 { "Every Day’s a Thursday" }
        p { "An entry with the id " mono { (id) } " does not appear to exist." }
    };

    wrappers::universal(body, headers, "missing_id", "Content not found")
}

pub fn section_error(headers: &HeaderMap, id: u32) -> Markup {
    let body = html! {
        h1 { "Every Day’s a Thursday" }
        p { "A section with the id " mono { (id) } " does not appear to exist." }
    };

    wrappers::universal(body, headers, "missing_id", "Content not found")
}
