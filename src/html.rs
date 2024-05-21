use axum::http::HeaderMap;
use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::{
    data::{Content, Entries, VolumeType, Volumes},
    routes::get_cookie,
};

fn universal(
    body: PreEscaped<String>,
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
        head {
            title { "Every Day’s a Thursday | " (title) }
            meta name="viewport" content="width=device-width, initial-scale=1, user-scalable=no";
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

pub fn setup(headers: &HeaderMap, content: &Content) -> Markup {
    let volumes_to_choose_from = content
        .volumes
        .values()
        .filter(|v| matches!(v.content_type, VolumeType::Journal));
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
                h2.volume { (PreEscaped(&volume.title)) }
                @for entry in volume.entries(&content) {
                    .entry edat-entry=(&entry.id) {
                        h3 { (PreEscaped(&entry.name)) }
                        p { (PreEscaped(&entry.description)) }
                    }
                }
            }
        }
        #configure {
            p { "Please intialize your user settings below. These can always be changed later." }
        }
    };
    universal(setup, headers, "setup", "Setup account")
}
