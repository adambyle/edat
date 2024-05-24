use axum::http::HeaderMap;
use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::{data, routes};

fn universal(
    body: PreEscaped<String>,
    headers: &HeaderMap,
    resource: &'static str,
    title: &str,
) -> Markup {
    let dark_theme = match routes::get_cookie(headers, "edat_theme") {
        Some("dark") => Some("dark-theme"),
        _ => None,
    };
    html! {
        (DOCTYPE)
        head {
            title { "Every Day’s a Thursday | " (title) }
            meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no";
            link type="text/css" rel="stylesheet" href={"style/" (resource) ".css"};
        }
        body class=[dark_theme] {
            (body)
            script src={"script/" (resource) ".js"} {};
            script src={"script/universal.js"} {};
        }
    }
}

pub fn login(headers: &HeaderMap) -> Markup {
    let login = html! {
        h1 { "Every Day’s a Thursday" }
        p { b { "Please enter your credentials below." } }
        p { "You should only need to do this once per device if cookies are enabled. Enter your name below (first or full) and your access code. Letter case does not matter." }
        ul {
            li {
                label for="name" { "NAME" }
                input #name-input name="name" type="text";
            }
            li {
                label for="code" { "ACCESS CODE" }
                input #code-input name="code" type="text";
            }
            li {
                button type="submit" onclick="login()" { "LOGIN" }
            }
        }
        p #error-msg style="display: none;" { "Invalid credentials." }
    };
    universal(login, headers, "login", "Login")
}

pub fn setup(headers: &HeaderMap, index: &data::Index) -> Markup {
    let volumes_to_choose_from = index
        .volumes()
        .filter(|v| v.content_type() == data::ContentType::Journal);
    let setup = html! {
        #welcome {
            h1 { "Every Day’s a Thursday" }
            p { "To improve your experience, the website makes recommendations based on your reading log." }
            p { "If the website ever recommends you something you have already read, please be willing to select \"" b {"I have already read this"} "\" to improve the log." }
            p { "Right now, your log does not exist. What would you like the recommendation system to know?" }
            ul {
                li { "If the system assumes you have read nothing, it will recommend you all the entries, including ones you have read before." }
                li { "If the system assumes you have read everything, it will only recommend you new releases after this point." }
                li { b { "Recommended: " } "You may also specify which entries you have already read so that the system can make precise recommendations. This process will take no longer than two minutes." }
            }
            p { b { "How would you like to begin?" } }
            button.record-choice #blank-record { "Assume I have read nothing" }
            button.record-choice #full-record { "Assume I have read everything" }
            button.record-choice #custom-record { "I will specify which entries I have read" }
        }
        #choose-entries {
            p { "Using the best of your knowledge, select the entries below that you believe you may have read before." }
            @for volume in volumes_to_choose_from {
                h2.volume { (PreEscaped(volume.title())) }
                @for entry in volume.entries(&index) {
                    .entry edat-entry=(entry.id()) {
                        h3 { (PreEscaped(entry.title())) }
                        p { (PreEscaped(entry.description())) }
                    }
                }
            }
        }
        #configure {
            p { b { "Your homepage is customizable to serve the most relevant content." } }
            p { "Select the elements below in the order (top to bottom) you would like them to appear on your homepage. You can include or omit whichever you want." }
            p { "Common resources, like the library, the index, and the addition history, will always have quick links at the top, but you can get more detailed information by selecting their widgets below." }
            .widget {
                span {}
                button #recent-widget {
                    h3 { "Recent additions" }
                    p { "Carousel of the latest sections" }
                }
            }
            .widget {
                span {}
                button #library-widget {
                    h3 { "The library" }
                    p { "Quick access to the main journal’s four books" }
                }
            }
            .widget {
                span {}
                button #conversations-widget {
                    h3 { "Conversations" }
                    p { "See where readers have recently commented" }
                }
            }
            .widget {
                span {}
                button #timeline-widget {
                    h3 { "The timeline" }
                    p { "Track your progress through the chronology" }
                }
            }
            .widget {
                span {}
                button #random-widget {
                    h3 { "Random entry" }
                    p { "Reading recommendation" }
                }
            }
            .widget {
                span {}
                button #extras-widget {
                    h3 { "Extras" }
                    p { "Quick access to old journals, fiction, and more" }
                }
            }
            .widget {
                span {}
                button #search-widget {
                    h3 { "Search bar" }
                    p { "Website search feature" }
                }
            }
            p { "You can always change these settings later." }
            button #done { "Finished" }

        }
    };
    universal(setup, headers, "setup", "Setup account")
}
