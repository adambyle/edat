use crate::search as search_tools;

pub mod search;

use super::*;

pub fn library_search(index: &Index, words: &[&str]) -> Markup {
    let mut results = Vec::new();

    let words = {
        let mut unique_words = Vec::with_capacity(words.len());
        for word in words {
            if !unique_words.contains(word) {
                unique_words.push(word);
            }
        }
        unique_words
    };

    for volume in index.volumes() {
        let hits = volume.search_index().words(&words);
        let title_hits = hits.for_section("TITLE").unwrap();
        let subtitle_hits = hits.for_section("SUBTITLE").unwrap();

        // No results in this volume.
        if hits.total_hit_count() == 0 {
            continue;
        }

        // Result in title or subtitle.
        if !title_hits.2.is_empty() || !subtitle_hits.2.is_empty() {
            let title = search_tools::bolden(volume.title(), &title_hits.2);
            let subtitle = volume
                .subtitle()
                .map(|s| search_tools::bolden(s, &subtitle_hits.2));
            results.push((
                hits.total_score(),
                hits.all_words_found(),
                html! {
                    .result {
                        p.label { "Collection" }
                        a.result-info href={ "/volume/" (volume.id()) } {
                            h4 { (PreEscaped(title)) }
                            @if let Some(subtitle) = subtitle {
                                p.details { (PreEscaped(subtitle)) }
                            }
                        }
                    }
                },
            ));
            continue;
        }

        // Result in intro.
        results.push((
            hits.total_score(),
            hits.all_words_found(),
            html! {
                .result {
                    p.label { "Collection — see intro" }
                    a.result-info href={ "/volume/" (volume.id()) } {
                        h4 { (PreEscaped(volume.title())) }
                        @if let Some(subtitle) = volume.subtitle() {
                            p.details { (PreEscaped(subtitle)) }
                        }
                    }
                }
            },
        ));
    }

    let mut total_section_hits = 0;
    for entry in index.entries() {
        let hits = entry.search_index().words(&words);
        let title_hits = hits.for_section("TITLE").unwrap();
        let description_hits = hits.for_section("DESCRIPTION").unwrap();
        let summary_hits = hits.for_section("SUMMARY").unwrap();

        // Get a score bonus from section results.
        let mut all_words_found_in_section = false;
        let section_score: f64 = entry
            .sections()
            .map(|s| {
                let results = s.search_index().words(&words);
                total_section_hits += results.total_hit_count();
                all_words_found_in_section |= results.all_words_found();
                results.total_score()
            })
            .sum();

        // No results in this entry.
        if title_hits.2.is_empty()
            && description_hits.2.is_empty()
            && summary_hits.2.is_empty()
            && section_score == 0.0
        {
            continue;
        }

        let details = if description_hits.0 > summary_hits.0 {
            search_tools::bolden(entry.description(), description_hits.2)
        } else {
            search_tools::bolden(entry.summary(), summary_hits.2)
        };

        let title = search_tools::bolden(entry.title(), title_hits.2);
        results.push((
            hits.total_score() + section_score,
            hits.all_words_found() || all_words_found_in_section,
            html! {
                .result {
                    p.label { "Entry in " (PreEscaped(entry.parent_volume().title())) }
                    a.result-info href={ "/entry/" (entry.id()) } {
                        h4 { (PreEscaped(title)) }
                        p.details { (PreEscaped(details)) }
                    }
                }
            },
        ));
    }

    results.sort_by(|r1, r2| match r2.1.cmp(&r1.1) {
        std::cmp::Ordering::Equal => r2.0.partial_cmp(&r1.0).unwrap(),
        ordering => ordering,
    });

    // A prompt for the user to see additional results.
    let search_prompt = html! {
        p.see-more { "See " (total_section_hits) " hits in entry content" }
        a.see-more-button href={"/search/" (words.join(","))} {
            "Go to full search"
        }
    };

    let mut results_html = Vec::with_capacity(2 + results.len());
    let mut lesser_result_notif_shown = false;
    let mut any_shown = false;
    for result in results {
        // If the result doesn't contain all the target words...
        if !lesser_result_notif_shown && !result.1 {
            lesser_result_notif_shown = true;
            if total_section_hits > 0 {
                results_html.push(search_prompt.clone());
            }
            results_html.push(if any_shown {
                html! {
                    p.lesser-results { "See also" }
                }
            } else {
                html! {
                    p.lesser-results { "No exact matches" }
                }
            });
        }

        results_html.push(result.2);
        any_shown = true;
    }

    if !lesser_result_notif_shown && total_section_hits > 0 {
        results_html.push(search_prompt);
    }

    if results_html.is_empty() {
        html! {
            p.no-results { "No results" }
        }
    } else {
        html! {
            @for result in results_html {
                (result)
            }
        }
    }
}

pub fn thread(user: &User, section: u32, line: usize) -> Markup {
    let index = user.index();
    
    let Ok(section) = index.section(section) else {
        return html! { "Error" }
    };

    let thread = section.comments(line);

    let comment_html = |comment: &Comment| {
        let author = comment.author.first_name();
        if !comment.show {
            return html! {
                .comment.hidden {
                    (author)
                    " has removed a message"
                    @if &comment.author == user {
                        .unremove edat_uuid=(comment.uuid) { "Unremove" }
                    }
                }
            };
        }

        html! {
            .comment {
                .text {
                    (PreEscaped(comment.content.last().unwrap()))
                }
                .info {
                    .author { (comment.author.first_name()) }
                    utc.date { (comment.timestamp) }
                    @if comment.content.len() > 1 {
                        .edited { "Edited" }
                    }
                    @if &comment.author == user {
                        .span .user-controls {
                            .remove edat_uuid=(comment.uuid) { "Remove" }
                            .edit edat_uuid=(comment.uuid) { "Edit" }
                        }
                    }
                }
            }
        }
    };
    
    let input = html! {
        #comment-input {
            #comment-instructions {
                "Reminder: Please make replying to readers a secondary goal."
            }
            textarea #user-comment placeholder="Say something about the entry text…" {}
        }
    };
    
    html! {
        #comments {
            @if thread.comments.len() == 0 {
                #no-comments.comment {
                    p { "No comments" }
                    p #close-comments { "✕" }
                }
            } @else {
                #some-comments.comment {
                    p { "Comments" }
                    p #close-comments { "✕" }
                }
                @for comment in thread.comments {
                    (comment_html(&comment))
                }
            }
        }
        (input)
    }
}

pub fn widget_options(widgets: &[String]) -> Markup {
    struct WidgetData {
        pub name: String,
        pub description: String,
        pub order: Option<usize>,
        pub id: String,
    }

    let widgets = {
        use WidgetData as W;

        let order = |id| widgets.iter().position(|s| s == id);

        vec![
            W {
                name: "Recent additions".to_owned(),
                description: "Carousel of the latest sections".to_owned(),
                order: order(&"recent-widget"),
                id: "recent-widget".to_owned(),
            },
            W {
                name: "The library".to_owned(),
                description: "Quick access to the main journal’s four books".to_owned(),
                order: order(&"library-widget"),
                id: "library-widget".to_owned(),
            },
            W {
                name: "Last read".to_owned(),
                description: "Return to where you left off".to_owned(),
                order: order(&"last-widget"),
                id: "last-widget".to_owned(),
            },
            W {
                name: "Conversations".to_owned(),
                description: "See where readers have recently commented".to_owned(),
                order: order(&"conversations-widget"),
                id: "conversations-widget".to_owned(),
            },
            W {
                name: "Reading recommendation".to_owned(),
                description: "Based on what you have left to read".to_owned(),
                order: order(&"random-widget"),
                id: "random-widget".to_owned(),
            },
            W {
                name: "Extras".to_owned(),
                description: "Quick access to old journals, fiction, and more".to_owned(),
                order: order(&"extras-widget"),
                id: "extras-widget".to_owned(),
            },
            W {
                name: "Global search".to_owned(),
                description: "Site-wide search for text content".to_owned(),
                order: order(&"search-widget"),
                id: "search-widget".to_owned(),
            },
        ]
    };

    html! {
        button.widget-select-all { "Select all" }
        @for widget in widgets {
            .widget {
                @if let Some(order) = widget.order {
                    span style="opacity: 1" { "#" (order + 1) }
                } @else {
                    span {}
                }
                button #(widget.id) .selected[widget.order.is_some()] {
                    h3 { (PreEscaped(&widget.name)) }
                    p { (PreEscaped(&widget.description)) }
                }
            }
        }
    }
}
