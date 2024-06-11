use maud::DOCTYPE;

use super::*;

pub(super) fn standard(body: Markup) -> Markup {
    html! {
        h1 #title { span { "Every Day’s a Thursday" } }
        main {
            nav #topnav {
                a href="/" { "HOME" }
                a href="/library" { "LIBRARY" }
                a href="/history" { "HISTORY" }
                a href="/forum" { "FORUM" }
                a href="/profile" { "PROFILE" }
            }
            (body)
        }
    }
}

pub(super) fn universal(body: Markup, headers: &HeaderMap, resource: &'static str, title: &str) -> Markup {
    let dark_theme = match get_cookie(headers, "edat_theme") {
        Some("dark") => Some("dark-theme"),
        _ => None,
    };

    html! {
        (DOCTYPE)
        html lang="en-us" {
            head {
                title { "Every Day’s a Thursday | " (title) }
                meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no";
                link type="text/css" rel="stylesheet" href={"style/" (resource) ".css"};
            }
            body class=[dark_theme] {
                (body)
                script type="module" src={"script/" (resource) ".js"} {};
            }
        }
    }
}
