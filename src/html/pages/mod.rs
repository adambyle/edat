use super::*;

pub mod home;
pub mod profile;
pub mod volume;

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
                button type="submit" id="login-button" { "LOGIN" }
            }
        }
        p #error-msg style="display: none;" { "Invalid credentials." }
    };
    wrappers::universal(login, headers, "login", "Login")
}

pub fn setup(headers: &HeaderMap, index: &Index) -> Markup {
    // TODO select all!
    
    let volumes = index
        .volumes()
        .filter(|v| v.kind() == crate::data::volume::Kind::Journal);

    let setup = html! {
        #welcome {
            h1 { "Every Day’s a Thursday" }
            p { "To improve your experience, the website makes recommendations based on your reading log." }
            p { "If the website ever recommends you something you have already read, please be willing to select \"" b {"Mark as read"} "\" to improve the log." }
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
            @for volume in volumes {
                h2.volume { (PreEscaped(volume.title())) }
                @for entry in volume.entries() {
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
            (components::widget_options(&[]))
            p { "You can always change these settings later." }
            button #done { "Finished" }
        }
    };
    wrappers::universal(setup, headers, "setup", "Setup account")
}

pub fn terminal(headers: &HeaderMap, allowed: bool) -> maud::Markup {
    let body = if allowed {
        html! {
            h1 { b { "Command terminal" } }
            input #command type="text"
                placeholder="Enter command here"
                autocomplete="off"
                autocapitalize="off"
                spellcheck="false"
                autofocus {}
            p #invalid-command style="opacity: 0.0" { "Invalid command" }
            #response {

            }
        }
    } else {
        html! {
            p #forbidden { "You do not have access to the terminal" }
        }
    };
    wrappers::universal(body, headers, "terminal", "Terminal")
}
