use std::fs;

use chrono::Utc;
use rand::prelude::SliceRandom;

use super::*;
use crate::data::volume::Kind as VolumeKind;

pub fn home<'index>(headers: &HeaderMap, user: &User) -> maud::Markup {
    let introduction = fs::read_to_string("content/edat.intro").unwrap();

    let widgets = user.widgets().iter().map(|w| match w.as_ref() {
        "recent-widget" => recent_widget(user),
        "library-widget" => library_widget(user),
        "last-widget" => last_widget(user),
        "conversations-widget" => conversations_widget(user),
        "random-widget" => random_widget(user),
        "extras-widget" => extras_widget(user),
        "search-widget" => search_widget(user.index()),
        _ => html! {},
    });

    let body = html! {
        #widgets-wrapper {
            #widgets {
                @for widget in widgets {
                    (widget)
                }
                @if user.widgets().is_empty() {
                    .widget #empty-widget {
                        h2 { "Customize your homepage" }
                        p { "You haven’t added any elements to your homepage yet, like quick access to recent entries or library shortcuts, but you can do so in your settings." }
                        a href="/profile" { "Go to settings" }
                    }
                }
            }
            .widget #intro-widget {
                h2 { "Introduction" }
                .introduction {
                    @for line in introduction.lines() {
                        p { (PreEscaped(line)) }
                    }
                }
            }
        }
    };
    let body = wrappers::standard(body, Vec::new(), None);
    wrappers::universal(body, &headers, "home", "Home", false)
}

fn recent_widget(user: &User) -> Markup {
    // Get whether the user wants to sections to show more details, by default.
    let expand_preference = user.preferences().get("expand_recents");
    let expand = match expand_preference {
        Some(preference) if preference == "false" => false,
        _ => true,
    };
    let detail_class = if expand {
        "show-detailed"
    } else {
        "show-concise"
    };

    // Get all the complete sections from journal volumes.
    let mut sections: Vec<_> = user
        .index()
        .sections()
        .filter(|s| {
            s.status() == section::Status::Complete
                && s.parent_entry().parent_volume().kind() == VolumeKind::Journal
        })
        .collect();

    // Sort them by recency.
    sections.sort_by_key(|s| (s.date(), s.index_in_parent()));
    sections.reverse();

    // Take only the first 10.
    sections.truncate(10);

    // Processes a section into html.
    let section_html = |section: &Section| {
        // Get the previous section.
        let index_in_parent = section.index_in_parent();
        let previous = section.parent_entry().sections().rev().find(|s| {
            s.status() == section::Status::Complete && s.index_in_parent() < index_in_parent
        });
        let omit_previous = previous.as_ref().is_some_and(|previous| {
            previous.status() != section::Status::Complete
                || sections.iter().any(|s| s.id() == previous.id())
        });
        let previous_html = if omit_previous {
            None
        } else {
            previous.map(|previous| {
                let prev_desc = html! {
                    p.previous-label {"Previous section"}
                    p.previous-description { (PreEscaped(previous.description())) }
                };
                if expand {
                    html! {
                        a.previous.detailed
                            href={"/section/" (previous.id())}
                        {
                            (prev_desc)
                        }
                    }
                } else {
                    html! {
                        a.previous.detailed
                            style="display: none"
                            href={"/section/" (previous.id())}
                        {
                            (prev_desc)
                        }
                    }
                }
            })
        };

        let last_read = user
            .history()
            .into_iter()
            .find(|(s, _)| s.id() == section.id())
            .map(|(_, h)| h.timestamp());

        let parent_volume = {
            let parent_volume = section.parent_entry().parent_volume();
            let parent_volume_title = parent_volume.title();
            if parent_volume.parts_count() == 1 {
                parent_volume_title.to_owned()
            } else {
                let part =
                    roman::to(section.parent_entry().parent_volume_part() as i32 + 1).unwrap();
                format!("{} vol. {}", parent_volume_title, part)
            }
        };

        let concise_desc = html! {
            p.description { (PreEscaped(section.description())) }
        };

        let detailed_desc = html! {
            p.summary { (PreEscaped(section.summary())) }
            span.details {
                span.index {
                    @if section.parent_entry().section_count() == 1 {
                        "Standalone"
                    } @else {
                        "Section " (1 + section.index_in_parent())
                    }
                }
                span.wordcount {
                    (section.length_string()) " words"
                }
                span.date {
                    "Added " (date_string(&section.date()))
                }
            }
        };

        html! {
            .section edat-unread[last_read.is_none()] {
                a.section-info href={ "/section/" (section.id()) } {
                    // Volume and entry header.
                    @if expand {
                        p.volume.detailed { (PreEscaped(parent_volume)) }
                    } @else {
                        p.volume.detailed
                            style="display: none"
                        { (PreEscaped(parent_volume)) }
                    }
                    h3 { (PreEscaped(section.parent_entry().title())) }

                    // Inner description
                    @if expand {
                        .concise style="display: none" {
                            (concise_desc)
                        }
                        .detailed {
                            (detailed_desc)
                        }
                    } @else {
                        .concise {
                            (concise_desc)
                        }
                        .detailed style="display: none" {
                            (detailed_desc)
                        }
                    }
                }

                @if let Some(ref time) = last_read {
                    span.read { "You read on " utc { (time) } }
                } @else {
                    span.unread-wrapper {
                        span.unread { "Unread" }
                        button.skip edat-section=(section.id()) { "Mark as read" }
                    }
                }
                @if let Some(previous_html) = previous_html {
                    (previous_html)
                }
            }
        }
    };

    html! {
        .widget #recent-widget {
            h2 { "Recent uploads" }
            #recent-carousel class=(detail_class) {
                @for section in &sections {
                    (section_html(section))
                }
                .section {
                    a .section-info.see-more href="/history" {
                       p { "See the full history" }
                    }
                }
            }
            button id="recent-expand" {
                @if expand {
                    span.detailed { "Hide details" }
                    span.concise style="display: none" { "Show details" }
                } @else {
                    span.detailed style="display: none" { "Hide details" }
                    span.concise { "Show details" }
                }
            }
        }
    }
}

fn library_widget(user: &User) -> Markup {
    let volumes = user
        .index()
        .volumes()
        .filter(|v| v.kind() == VolumeKind::Journal);

    html! {
        .widget #library-widget {
            h2 { "The library" }
            .volumes {
                @for volume in volumes {
                    a.volume href={ "/volume/" (volume.id()) } {
                        h3 { (PreEscaped(volume.title())) }
                        @if let Some(subtitle) = volume.subtitle() {
                            p.subtitle { (PreEscaped(subtitle)) }
                        }
                    }
                }
                a.volume-link href="/library" { "Search the full library" }
            }
        }
    }
}

fn last_widget(user: &User) -> Markup {
    let section = user.history().into_iter().find(|(s, h)| {
        s.parent_entry().parent_volume().kind() == VolumeKind::Journal
            && !matches!(h, SectionProgress::Finished { .. })
    });

    let section = if let Some((ref section, ref progress)) = section {
        let progress_pp = (progress.line() as f32 / section.lines() as f32 * 100.0).round();
        html! {
            a.see-profile href="/profile" {
                "See reading history in your profile"
            }
            a.last-section href={ "/section/" (section.id()) "?line=" (progress.line()) } {
                h3 { (PreEscaped(section.parent_entry().title())) }
                p.summary {
                    (PreEscaped(section.summary()))
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
    } else {
        html! {
            .last-section.nothing {
                p { "You have no unfinished reading to pick up on." }
            }
        }
    };

    html! {
        .widget #last-widget {
            h2 { "Last read" }
            (section)
        }
    }
}

fn conversations_widget(user: &User) -> Markup {
    let index = user.index();

    let now = Utc::now().timestamp();
    let one_month = 30 * 24 * 60 * 60;

    let mut threads = Vec::new();

    let sections: Vec<_> = index.sections().collect();

    for section in &sections {
        let section_threads = section.threads();

        for thread in section_threads {
            if thread
                .comments
                .iter()
                .any(|c| now - c.timestamp <= one_month && c.show)
            {
                threads.push((section, thread));
            }
        }
    }

    threads.sort_by_key(|(_, t)| t.comments.iter().map(|c| c.timestamp).max().unwrap());
    threads.reverse();

    let threads_html: Vec<_> = threads
        .iter()
        .map(|(s, t)| {
            let content = s.content();
            let comment = t.comments.iter().rev().find(|c| c.show).unwrap();
            let mut comment_text = comment.content.last().unwrap().to_owned();
            if comment_text.len() > 150 {
                comment_text = format!("{}…", &comment_text[..150]);
            }

            struct ThreadLine {
                text: String,
                in_note: bool,
            }

            let mut thread_lines = Vec::new();
            let mut in_aside = false;
            let mut in_note = false;

            let mut target_index: usize = 0;
            let mut working_index = 0;

            for (i, line) in content.lines().enumerate() {
                if line.starts_with("/end") {
                    in_aside = false;
                    in_note = false;
                    continue;
                }
                if line.starts_with("/img") {
                    continue;
                }
                if line.starts_with("/aside") {
                    in_aside = true;
                }
                if in_aside {
                    continue;
                }
                if line.starts_with("/note") {
                    in_note = true;
                    continue;
                }

                thread_lines.push(ThreadLine {
                    text: line.to_owned(),
                    in_note,
                });
                if i == t.line {
                    target_index = working_index;
                }
                working_index += 1;
            }

            let mut line_html = Vec::with_capacity(thread_lines.len());

            let mut in_note = false;
            for (i, line) in thread_lines.into_iter().enumerate() {
                if i >= target_index.saturating_sub(4) && i <= target_index + 4 {
                    if line.in_note != in_note {
                        in_note = line.in_note;
                        if in_note {
                            line_html.push(html! { (PreEscaped("<div class=\"note\">")) });
                        } else {
                            line_html.push(html! { (PreEscaped("</div>")) });
                        }
                    }
                    line_html.push(html! {
                        @if i == target_index {
                            .line.highlight { (PreEscaped(line.text)) }
                        } @else {
                            .line { (PreEscaped(line.text)) }
                        }
                    });
                }
            }
            if in_note {
                line_html.push(html! { (PreEscaped("</div>")) });
            }

            html! {
                a.thread href={ "/section/" (s.id()) "?line=" (t.line) } {
                    p.title {
                        (PreEscaped(s.parent_entry().title()))
                        @if s.parent_entry().section_count() > 1 {
                            span.index {
                                "Section " (1 + s.index_in_parent())
                            }
                        }
                    }
                    .body {
                        @for line in line_html {
                            (line)
                        }
                    }
                    p.comments {
                        @if t.comments.len() > 1 {
                            .more {
                                (t.comments.len() - 1) " others"
                            }
                        }
                        .comment {
                            .text {
                                (PreEscaped(comment_text))
                            }
                            .info {
                                .author { (comment.author.first_name()) }
                                utc.date { (comment.timestamp) }
                                @if comment.content.len() > 1 {
                                    .edited { "Edited" }
                                }
                            }
                        }
                    }
                }
            }
        })
        .collect();

    html! {
        .widget #conversations-widget {
            h2 { "Comments" }

            @if threads_html.is_empty() {
                .no-threads { "No recent comments" }
            } @else {
                #threads-carousel {
                    @for thread in threads_html {
                        (thread)
                    }
                }
            }
        }
    }
}

fn random_widget(user: &User) -> Markup {
    let entry_html = |entry: &Entry, url: String| {
        let parent_volume = {
            let parent_volume = entry.parent_volume();
            let parent_volume_title = parent_volume.title();
            if parent_volume.parts_count() == 1 {
                parent_volume_title.to_owned()
            } else {
                let part = roman::to(entry.parent_volume_part() as i32 + 1).unwrap();
                format!("{} vol. {}", parent_volume_title, part)
            }
        };
        html! {
            a.entry href=(url) {
                p.volume { (PreEscaped(parent_volume)) }
                h3 { (PreEscaped(entry.title())) }
                p.summary { (PreEscaped(entry.summary())) }
            }
        }
    };

    let entry = 'entry: {
        // Collect all entries that are finished and shuffle them.
        let mut entries: Vec<_> = user
            .index()
            .entries()
            .filter(|e| {
                e.section_count() > 0
                    && !e
                        .sections()
                        .any(|s| s.status() != section::Status::Complete)
                    && e.parent_volume().kind() == VolumeKind::Journal
            })
            .collect();
        entries.shuffle(&mut rand::thread_rng());

        // Try to find an unstarted entry.
        let mut started_entries = Vec::with_capacity(entries.len());
        for entry in entries {
            match user.entry_progress(&entry) {
                None => {
                    // We found an unstarted entry.
                    let url = format!("/entry/{}", entry.id());
                    break 'entry html! {
                        (entry_html(&entry, url))
                        p.label { "You haven’t started this entry" }
                    };
                }
                Some(progress) => {
                    // Check this one later.
                    started_entries.push((entry, progress));
                }
            }
        }

        // Try to find an unfinished entry.
        let mut finished_entries = Vec::with_capacity(started_entries.len());
        for (entry, progress) in started_entries {
            match progress {
                EntryProgress::UpToSection {
                    section_id,
                    section_index,
                    ..
                } => {
                    let url = format!("/section/{section_id}");
                    break 'entry html! {
                        (entry_html(&entry, url))
                        p.label { "You need to start section " (section_index + 1) }
                    };
                }
                EntryProgress::InSection {
                    section_id,
                    section_index,
                    line,
                    ..
                } => {
                    let url = format!("/section/{section_id}?line={line}");
                    break 'entry html! {
                        (entry_html(&entry, url))
                        p.label { "You’re partway through section " (section_index + 1) }
                    };
                }
                EntryProgress::Finished { last_read } => finished_entries.push((entry, last_read)),
            }
        }

        // Otherwise, just pick a random entry.
        let (entry, last_read) = finished_entries.last().unwrap();
        let url = format!("/entry/{}", entry.id());
        html! {
            (entry_html(entry, url))
            p.label { "You haven’t read this since " utc { (last_read) } }
        }
    };

    html! {
        .widget #random-widget {
            h2 { "Reading recommendation" }
            (entry)
        }
    }
}

fn extras_widget(user: &User) -> Markup {
    let volumes = user
        .index()
        .volumes()
        .filter(|v| v.kind() != VolumeKind::Journal && v.kind() != VolumeKind::Featured);

    html! {
        .widget #extras-widget {
            h2 { "Extras" }
            .volumes {
                @for volume in volumes {
                    a.volume href={ "/volume/" (volume.id()) } {
                        h3 { (PreEscaped(volume.title())) }
                        @if let Some(subtitle) = volume.subtitle() {
                            p.subtitle { (PreEscaped(subtitle)) }
                        }
                    }
                }
                a.volume-link href="/library" { "Search the full library" }
            }
        }
    }
}

fn search_widget(index: &Index) -> Markup {
    let word_total = index
        .sections()
        .map(|s| s.search_index().total_word_count())
        .sum::<usize>()
        + index
            .entries()
            .map(|e| e.search_index().total_word_count())
            .sum::<usize>()
        + index
            .volumes()
            .map(|v| v.search_index().total_word_count())
            .sum::<usize>();

    html! {
        .widget #search-widget {
            h2 { "Global search" }
            input #search-input type="text" edat_total=(word_total) placeholder={
                "Search 0 words of content"
            };
            p #search-footer { "Enter a word or series of words separated by spaces" }
        }
    }
}
