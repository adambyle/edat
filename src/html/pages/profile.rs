use chrono::Utc;

use super::*;

/// Profile page for a given user.
pub fn profile(headers: &HeaderMap, user: &User) -> Markup {
    let history_preview_length = 3;

    // Get the sections read in the last two months.
    let two_months_ago = Utc::now().timestamp() - 60 * 60 * 24 * 60;
    let mut sections = user
        .history()
        .into_iter()
        .filter(|(_, h)| h.timestamp() > two_months_ago);

    // The preview will be just 3 sections. The user can expand the rest.
    let history_preview: Vec<_> = sections.by_ref().take(history_preview_length).collect();
    let history_rest: Vec<_> = sections.by_ref().skip(history_preview_length).collect();

    let profile = html! {
        h1 { a href="/" { "Every Day’s a Thursday" } }
        #homepage.module {
            h2 { "Homepage settings"}
            .wrapper {
                p { "Choose which widgets to include on your homepage. Changes are saved automatically. The order you select them in will determine the order they appear on your homepage." }
                #widgets {
                    (components::widget_options(user.widgets()))
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
                        (history_entry(section))
                    }
                }
                #history-rest {
                    @for section in history_rest {
                        (history_entry(section))
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

    wrappers::universal(profile, headers, "profile", "Profile", false)
}

fn history_entry((section, progress): (Section, SectionProgress)) -> Markup {
    let progress_pp = if progress.line() == 0 {
        100.0
    } else {
        (progress.line() as f32 / section.lines() as f32 * 100.0).round()
    };

    html! {
        a.section href={ "/section/" (section.id()) "?line=" (progress.line()) } {
            h3 { (PreEscaped(section.parent_entry().title())) }
            p.description {
                (PreEscaped(section.description()))
                span.index {
                    (1 + section.index_in_parent())
                    "/"
                    (section.parent_entry().section_count())
                }
            }
            p.info {
                span.progress { (progress_pp) "% complete" }
                span.lastread { "Last read " utc { (progress.timestamp()) } }
            }
        }
    }
}
