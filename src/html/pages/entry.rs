use super::*;

pub enum EntryDestination {
    Top,
    Section(u32),
    Line(u32, usize),
}

pub fn entry(headers: &HeaderMap, entry: &Entry, user: &User) -> Markup {
    let body = html! {
        h2.page-title { (PreEscaped(entry.title())) }
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
