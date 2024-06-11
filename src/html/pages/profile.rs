use super::*;

pub fn profile(headers: &HeaderMap, data: profile::ProfileData) -> Markup {
    let widget_data = widgets::ordered_widget_data(&data.widgets);

    let history_preview_length = 3;
    let history_preview = data.sections.iter().take(history_preview_length);
    let history_rest = data.sections.iter().skip(history_preview_length);

    let profile = html! {
        h1 { a href="/" { "Every Day’s a Thursday" } }
        #homepage.module {
            h2 { "Homepage settings"}
            .wrapper {
                p { "Choose which widgets to include on your homepage. Changes are saved automatically. The order you select them in will determine the order they appear on your homepage." }
                #widgets {
                    (widgets::widget_options_component(widget_data))
                }
            }
            p.expand #homepage-expand { "Show options" }
        }
        #history.module {
            h2 { "Reading history" }
            .wrapper {
                p { "Only the last two months are shown. Select a section below to return to your place." }
                #history-preview {
                    @for section in history_preview {
                        (section.html())
                    }
                }
                #history-rest {
                    @for section in history_rest {
                        (section.html())
                    }
                }
            }
            p.expand #history-expand { "Show more" }
        }
        #contributions.module {
            h2 { "Contributions" }
            p { "The ability to write featured content is coming soon, including the Perspectives feature
                and Personal Journals. Your hub for writing and editing will be right here." }
        }
        button #home { "Go back" }
    };

    wrappers::universal(profile, headers, "profile", "Profile")
}

pub struct ProfileData {
    pub widgets: Vec<String>,
    pub sections: Vec<ViewedSection>,
}

pub struct ViewedSection {
    pub id: u32,
    pub description: String,
    pub timestamp: i64,
    pub entry: String,
    pub index: (usize, usize),
    pub progress: (usize, usize),
}

impl ViewedSection {
    pub fn html(&self) -> Markup {
        let progress = if self.progress.0 == 0 {
            100.0
        } else {
            (self.progress.0 as f32 / self.progress.1 as f32 * 100.0).round()
        };

        html! {
            a.section href={ "/section/" (self.id) "?line=" (self.progress.0) } {
                h3 { (PreEscaped(&self.entry)) }
                p.description {
                    (PreEscaped(&self.description))
                    span.index { (self.index.0 + 1) "/" (self.index.1) }
                }
                p.info {
                    span.progress { (progress) "% complete" }
                    span.lastread { "Last read " utc { (self.timestamp) } }
                }
            }
        }
    }
}
