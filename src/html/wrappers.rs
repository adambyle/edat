use maud::DOCTYPE;

use super::*;

pub(super) fn standard(body: Markup, drawers: Vec<Markup>) -> Markup {
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
            #drawer {
                p.notification {
                    span.text { }
                    span.open { "ߍ" }
                }
                @for drawer in drawers {
                    (drawer)
                }
            }
        }
    }
}

pub(super) fn universal(
    body: Markup,
    headers: &HeaderMap,
    resource: &'static str,
    title: &str,
) -> Markup {
    let dark_theme = match get_cookie(headers, "edat_theme") {
        Some("dark") => Some("dark-theme"),
        _ => None,
    };

    html! {
        (DOCTYPE)
        html lang="en-us" {
            head {
                title { "Every Day’s a Thursday | " (crate::data::strip_formatting(title)) }
                meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no";
                link type="text/css" rel="stylesheet" href={"/style/" (resource) ".css"};
                @if dark_theme.is_some() {
                    link rel="icon" type="image/x-icon" href="/asset/favicon-dark.ico";
                    link rel="icon" type="image/png" sizes="16x16" href="/asset/favicon16-dark.png";
                    link rel="icon" type="image/png" sizes="32x32" href="/asset/favicon32-dark.png";
                    link rel="apple-touch-icon" href="/asset/apple-touch-icon-dark.png";
                } @else {
                    link rel="icon" type="image/x-icon" href="/asset/favicon-light.ico";
                    link
                        rel="icon"
                        type="image/png"
                        sizes="16x16"
                        href="/asset/favicon16-light.png";
                    link
                        rel="icon"
                        type="image/png"
                        sizes="32x32"
                        href="/asset/favicon32-light.png";
                    link rel="apple-touch-icon" href="/asset/apple-touch-icon-light.png";
                }
            }
            body class=[dark_theme] {
                (body)
                script type="module" src={"/script/" (resource) ".js"} {};
            }
        }
    }
}
