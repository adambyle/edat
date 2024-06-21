use super::*;

pub fn history(headers: &HeaderMap, user: &User) -> Markup {
    let mut sections: Vec<_> = user
        .index()
        .sections()
        .filter(|s| s.status() == section::Status::Complete)
        .collect();
    sections.sort_by_key(|s| (s.date(), s.index_in_parent()));
    sections.reverse();

    let mut sections_html = Vec::with_capacity(sections.len());
    let mut entry_title = None;
    for section in sections {
        let show_title = match entry_title {
            Some(ref title) => title != section.parent_entry().title(),
            None => true,
        };
        if show_title {
            entry_title = Some(section.parent_entry().title().to_owned());
        }

        let unread = user.section_progress(&section).is_none();

        sections_html.push(html! {
            a.section .new-entry[show_title] href={ "/section/" (section.id()) } {
                @if show_title {
                    h3 { (PreEscaped(section.parent_entry().title())) }
                }
                p.description { (PreEscaped(section.description())) }
                p.info {
                    @if section.parent_entry().section_count() > 1 {
                        span.section-index { "Section " (1 + section.index_in_parent()) }
                    } @else {
                        span.section-index { "Standalone" }
                    }
                    span.word_count { (section.length_string()) " words" }
                    span.date { "Added "(crate::data::date_string(&section.date())) }
                    @if unread {
                        span.unread { "Unread" }
                    }
                }
            }
        });
    }
    
    let body = html! {
        h2 { "Upload history" }
        #sections {
            @for section in sections_html {
                (section)
            }
        }
    };

    let body = wrappers::standard(body, Vec::new(), None);

    wrappers::universal(body, headers, "history", "Upload history")
}
