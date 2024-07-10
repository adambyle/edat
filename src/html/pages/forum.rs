use super::*;

pub fn forum(headers: &HeaderMap, _user: &User) -> Markup {
    let body = html! {
        h1 { "Every Day’s a Thursday" }
        p { "This page is under construction."}
        a href="/" { "Return home" }
    };

    wrappers::universal(body, headers, "forum", "Forum", false)
}
