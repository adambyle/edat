use super::*;

pub fn entry(headers: &HeaderMap, entry: &Entry, user: &User) -> Markup {
    todo!()
}

pub fn error(headers: &HeaderMap, id: &str) -> Markup {
    let body = html! {
        h1 { "Every Day’s a Thursday" }
        p { "An entry with the id " mono { (id) } " does not appear to exist." }
    };

    wrappers::universal(body, headers, "missing_id", "Content not found")
}
