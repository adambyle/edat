use std::collections::HashMap;

use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};

use super::*;

pub fn entry(index: &Index, id: &str, words: &[&str]) -> Markup {
    let Ok(entry) = index.entry(id.to_owned()) else {
        return error();
    };

    let search_results = entry.search_index().words(words);
    let title_hits = search_results.for_section("TITLE").unwrap();
    let description_hits = search_results.for_section("DESCRIPTION").unwrap();
    let summary_hits = search_results.for_section("SUMMARY").unwrap();

    let title = search_tools::bolden(entry.title(), &title_hits.2);
    let description = search_tools::bolden(entry.description(), &description_hits.2);
    let summary = search_tools::bolden(entry.summary(), &summary_hits.2);

    html! {
        .go {
            a href={"/entry/" (id)} { "Go to entry" }
        }
        .entry-body {
            .label { "Title" }
            .title { (PreEscaped(title)) }
            @if entry.parent_volume().kind() == crate::data::volume::Kind::Journal {
                .label { "Summary" }
                .summary { (PreEscaped(summary)) }
                .label { "Description" }
                .description { (PreEscaped(description)) }
            }
        }
    }
}

pub fn intro(index: &Index, id: &str, words: &[&str]) -> Markup {
    let Ok(volume) = index.volume(id.to_owned()) else {
        return error();
    };

    let search_results = volume.search_index().words(words);
    let intro_hits = search_results.for_section("INTRO").unwrap();

    let intro = search_tools::bolden(&volume.intro(), &intro_hits.2);
    let lines = intro.lines();

    html! {
        .go {
            a href={"/volume/" (id)} { "Go to collection" }
        }
        .intro-body {
            .label { "Introduction" }
            @for line in lines {
                p.line { (PreEscaped(line)) }
            }
        }
    }
}

pub fn section(index: &Index, id: u32, words: &[&str]) -> Markup {
    let Ok(section) = index.section(id) else {
        return error();
    };

    let stemmer = Stemmer::create(Algorithm::English);

    let search_results = section.search_index().words(words);
    let summary_hits = search_results.for_section("SUMMARY").unwrap();
    let heading_hits = search_results.for_section("HEADING").unwrap();
    let content_hits = search_results.for_section("CONTENT").unwrap();

    let summary = search_tools::bolden(section.summary(), &summary_hits.2);
    let heading = section
        .heading()
        .map(|h| search_tools::bolden(h, &heading_hits.2));
    let content = search_tools::bolden(&section.content(), &content_hits.2);

    let mut any_all_match = false;
    let lines: Vec<_> = content
        .lines()
        .enumerate()
        .map(|(line_index, line)| {
            if line.starts_with("/note") {
                let note_desc = line.split("/note ").nth(1);
                return PreEscaped(if let Some(note_desc) = note_desc {
                    format!(r#"<div class="note"><p class="note-desc">{note_desc}</p>"#)
                } else {
                    format!(r#"<div class="note">"#)
                });
            }
            if line.starts_with("/end") {
                return PreEscaped("</div>".to_string());
            }
            if line.starts_with("/aside") {
                return PreEscaped(r#"<div class="aside">"#.to_string());
            }

            if !line.contains("<b>") {
                return html! {
                    p.line.empty { "…" }
                };
            }

            if line.starts_with("/img") {
                let mut parts = line.split(" ").skip(1);
                let url = parts
                    .next()
                    .unwrap_or("")
                    .replace("<b>", "")
                    .replace("</b>", "");
                let caption = parts.collect::<Vec<_>>().join(" ");
                return html! {
                    .img {
                        img src={ "/image/" (url) ".jpg" };
                        p.caption { (PreEscaped(caption)) }
                    }
                };
            }

            let mut all_match = false;
            if words.len() > 1 {
                let mut found_words: HashMap<_, _> = words
                    .iter()
                    .map(|w| (stemmer.stem(*w).into_owned().to_lowercase(), false))
                    .collect();

                // Get boldened words.
                let bold_regex = Regex::new(r"<b>(.*?)</b>").unwrap();
                for instance in bold_regex.find_iter(&line) {
                    let instance = stemmer
                        .stem(&instance.as_str().replace("’", "'")[3..instance.len() - 4])
                        .to_lowercase();

                    // Mark this boldened word as found.
                    for (w, found) in &mut found_words {
                        if &instance == w {
                            *found = true;
                            break;
                        }
                    }

                    // If they're all found, mark this line as the one
                    // where all are marked.
                    if found_words.values().all(|&f| f) {
                        all_match = true;
                        break;
                    }
                }
            }

            any_all_match |= all_match;

            html! {
                a
                    .line
                    edat_line=(line_index)
                    href={"/section/" (section.id()) "?line=" (line_index)}
                    .allmatch[all_match]
                {
                    (PreEscaped(line))
                }
                @if all_match {
                    p.jump { "Jump to next full match" }
                }
            }
        })
        .collect();

    html! {
        .go {
            a href={"/section/" (id)} { "Go to section" }
            a href={"/entry/" (section.parent_entry_id())} { "Go to entry" }
        }
        .section-body {
            @if any_all_match {
                p.jump { "Jump to first full match" }
            }
            @if summary_hits.2.len() > 0 {
                .label { "Summary" }
                .summary { (PreEscaped(summary)) }
            }
            @if let Some(heading) = heading {
                .heading { (PreEscaped(heading)) }
            }
            @for line in lines {
                (line)
            }
        }
    }
}

pub fn volume(index: &Index, id: &str, words: &[&str]) -> Markup {
    let Ok(volume) = index.volume(id.to_owned()) else {
        return error();
    };

    let search_results = volume.search_index().words(words);
    let title_hits = search_results.for_section("TITLE").unwrap();
    let subtitle_hits = search_results.for_section("SUBTITLE").unwrap();

    let title = search_tools::bolden(volume.title(), &title_hits.2);
    let subtitle = volume
        .subtitle()
        .map(|s| search_tools::bolden(s, &subtitle_hits.2));

    html! {
        .go {
            a href={"/volume/" (id)} { "Go to collection" }
        }
        .volume-body {
            .label { "Title" }
            .title { (PreEscaped(title)) }
            @if let Some(subtitle) = subtitle {
                .label { "Subtitle" }
                .subtitle { (PreEscaped(subtitle)) }
            }
        }
    }
}

fn error() -> Markup {
    html! { "Error" }
}
