use super::*;
use crate::search::bolden;

pub fn search(headers: &HeaderMap, index: &Index, words: &[&str]) -> Markup {
    let words = {
        let mut unique_words = Vec::with_capacity(words.len());
        for word in words {
            if !unique_words.contains(word) {
                unique_words.push(word);
            }
        }
        unique_words
    };

    struct Result {
        score: f64,
        all_found: bool,
        widget: Markup,
        subresults: Vec<Subresult>,
        name: String,
    }

    #[derive(Debug)]
    struct Subresult {
        score: f64,
        all_found: bool,
        widget: Markup,
    }

    let mut results = Vec::new();
    let mut total_hits = 0;

    for volume in index.volumes() {
        let search_results = volume.search_index().words(&words);
        let title_hits = search_results.for_section("TITLE").unwrap();
        let subtitle_hits = search_results.for_section("SUBTITLE").unwrap();
        let intro_hits = search_results.for_section("INTRO").unwrap();

        if search_results.total_hit_count() == 0 {
            continue;
        }

        let mut subresults = Vec::new();

        if title_hits.2.len() > 0 || subtitle_hits.2.len() > 0 {
            let hits = title_hits.2.len() + subtitle_hits.2.len();
            subresults.push(Subresult {
                score: title_hits.0 + subtitle_hits.0,
                all_found: title_hits.1 || subtitle_hits.1,
                widget: html! {
                    .subresult.meta edat_search={"volume/" (volume.id())} {
                        p.name { "Collection info" }
                        p.hits { (hits) " hits" }
                    }
                },
            });
        }

        if intro_hits.2.len() > 0 {
            let hits = intro_hits.2.len();
            subresults.push(Subresult {
                score: intro_hits.0,
                all_found: intro_hits.1,
                widget: html! {
                    .subresult.meta edat_search={"intro/" (volume.id())} {
                        p.name { "Introduction" }
                        p.hits { (hits) " hits" }
                    }
                },
            });
        }

        total_hits += search_results.total_hit_count();

        let title = search::bolden(volume.title(), &title_hits.2);
        let subtitle = volume
            .subtitle()
            .map(|s| search::bolden(s, &subtitle_hits.2));

        subresults.sort_by(|r1, r2| match r2.all_found.cmp(&r1.all_found) {
            O::Equal => r2.score.partial_cmp(&r1.score).unwrap(),
            ordering => ordering,
        });

        results.push(Result {
            score: search_results.total_score(),
            all_found: search_results.all_words_found(),
            widget: html! {
                .result {
                    p.label { "Collection" }
                    h4 { (PreEscaped(title)) }
                    @if let Some(subtitle) = subtitle {
                        p.details { (PreEscaped(subtitle)) }
                    }
                    p.hits { (search_results.total_hit_count()) " hits" }
                    .mysubresults {
                        @for subresult in &subresults {
                            (subresult.widget)
                        }
                    }
                }
            },
            subresults,
            name: volume.title().to_string(),
        });
    }

    for entry in index.entries() {
        let search_results = entry.search_index().words(&words);
        let title_hits = search_results.for_section("TITLE").unwrap();
        let description_hits = search_results.for_section("DESCRIPTION").unwrap();
        let summary_hits = search_results.for_section("SUMMARY").unwrap();

        let sections: Vec<_> = entry
            .sections()
            .filter(|s| s.status() != section::Status::Missing)
            .map(|s| {
                let results = s.search_index().words(&words);
                (s, results)
            })
            .collect();
        let any_section_hits = sections.iter().any(|(_, s)| s.total_hit_count() > 0);

        if search_results.total_hit_count() == 0 && !any_section_hits {
            continue;
        }

        let mut subresults = Vec::new();

        if title_hits.2.len() > 0 || description_hits.2.len() > 0 || summary_hits.2.len() > 0 {
            let hits = title_hits.2.len() + description_hits.2.len() + summary_hits.2.len();
            subresults.push(Subresult {
                score: title_hits.0 + description_hits.0 + summary_hits.0,
                all_found: title_hits.1 || description_hits.1 || summary_hits.1,
                widget: html! {
                    .subresult.meta edat_search={"entry/" (entry.id())} {
                        p.name { "Entry info" }
                        p.hits { (hits) " hits" }
                    }
                },
            });
        }

        for (section, section_hits) in &sections {
            if section_hits.total_hit_count() == 0 {
                continue;
            }

            let description = search::bolden(
                section.description(),
                &section_hits.for_section("DESCRIPTION").unwrap().2,
            );

            let hits = section_hits.total_hit_count();
            subresults.push(Subresult {
                score: section_hits.total_score(),
                all_found: section_hits.all_words_found(),
                widget: html! {
                    .subresult.section edat_search={"section/" (section.id())} {
                        @if entry.section_count() == 1 {
                            p.section-index { "Standalone" }
                        } @else {
                            p.section-index { "Section " (1 + section.index_in_parent()) }
                        }
                        p.details { (PreEscaped(description)) }
                        p.hits { (hits) " hits" }
                    }
                },
            });
        }

        let hits = search_results.total_hit_count()
            + sections
                .iter()
                .map(|(_, s)| s.total_hit_count())
                .sum::<usize>();
        total_hits += hits;

        let title = search::bolden(entry.title(), &title_hits.2);
        let description = search::bolden(entry.description(), &description_hits.2);

        subresults.sort_by(|r1, r2| match r2.all_found.cmp(&r1.all_found) {
            O::Equal => r2.score.partial_cmp(&r1.score).unwrap(),
            ordering => ordering,
        });

        results.push(Result {
            score: search_results.total_score()
                + sections.iter().map(|(_, s)| s.total_score()).sum::<f64>(),
            all_found: search_results.all_words_found(),
            widget: html! {
                .result {
                    p.label { "Entry in " (PreEscaped(entry.parent_volume().title())) }
                    h4 { (PreEscaped(title)) }
                    p.details { (PreEscaped(description)) }
                    p.hits { (hits) " hits" }
                    .mysubresults {
                        @for subresult in &subresults {
                            (subresult.widget)
                        }
                    }
                }
            },
            subresults,
            name: entry.title().to_string(),
        });
    }

    use std::cmp::Ordering as O;
    results.sort_by(|r1, r2| match r2.all_found.cmp(&r1.all_found) {
        O::Equal => r2.score.partial_cmp(&r1.score).unwrap(),
        ordering => ordering,
    });

    let body = html! {
        h1 { a href="/" { "Every Day’s a Thursday" } }
        #search-wrapper {
            p.hit-count {
                @if words.len() > 0 {
                    (total_hits) " hits for"
                } @else {
                    "Search for a word or phrase below"
                }
            }
            input #search-input type="text" value={(words.join(" "))};
        }
        @if !results.is_empty() {
            .carousel-header {
                "Choose an entry or collection"
            }
            .results.carousel {
                @for result in &results {
                    (result.widget)
                }
            }
            .carousel-header {
                "Results in “"
                span #result-name { (PreEscaped(&results[0].name)) }
                "”"
            }
            .subresults.carousel {
                @for subresult in &results[0].subresults {
                    (subresult.widget)
                }
            }
            .body {
                p.loading { "Loading results" }
            }
        }
    };

    wrappers::universal(body, headers, "search", "Search")
}
