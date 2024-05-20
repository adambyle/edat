use maud::{html, PreEscaped, DOCTYPE};

fn universal(body: PreEscaped<String>, resource: &'static str, title: &str) -> String {
    html! {
        (DOCTYPE)
        head {
            title { "Every Day’s a Thursday | " (title) }
            meta name="viewport" content="width=device-width, initial-scale=1";
            link type="text/css" rel="stylesheet" href={"style/" (resource) ".css"};
        }
        body {
            (body)
            script src={"script/" (resource) ".js"} {};
        }
    }
    .into_string()
}

pub fn login() -> String {
    let login = html! {
        h1 { "Every Day’s a Thursday" }
        p { b { "Please enter your credentials below." } }
        p { "You should only need to do this once per device if cookies are enabled. Enter your name below (first or full) and your access code. Letter case does not matter." }
        ul {
            li {
                label for="name" { "NAME" }
                input name="name" id="name-input" type="text";
            }
            li {
                label for="code" { "ACCESS CODE" }
                input name="code" id="code-input" type="text";
            }
            li {
                button type="submit" onclick="login()" { "LOGIN" }
            }
        }
        p id="error-msg" style="display: none;" { "Invalid credentials." }
    };
    universal(login, "login", "Login")
}
