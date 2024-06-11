use std::fs;

use html::pages::home::Widget;
use rand::Rng;

use super::*;

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {
    use html::pages::home as home_html;
    
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    // Initialize homepage widgets.
    let mut widgets = Vec::new();

    let recent_widget = || {
        // Get all the complete sections from journal volumes.
        let mut sections: Vec<_> = index
            .sections()
            .filter(|s| {
                let parent_entry = s.parent_entry();
                let parent_volume = parent_entry.parent_volume();
                parent_volume.kind() == volume::Kind::Journal
                    && s.status() == section::Status::Complete
            })
            .collect();

        // Sort them by recency.
        sections.sort_by(|a, b| {
            let date_a = a.date();
            let date_b = b.date();
            if date_a == date_b {
                b.id().cmp(&a.id())
            } else {
                date_b.cmp(&date_a)
            }
        });

        // Process the 10 latest sections...
        let recents = sections[..10.min(sections.len())]
            .iter()
            .map(|section| {
                let parent_entry = section.parent_entry();
                let in_entry = section.index_in_parent();
                let previous =
                    (in_entry > 0).then(|| parent_entry.sections().nth(in_entry - 1).unwrap());
                let read = user
                    .history()
                    .iter()
                    .find(|(s, _)| s.id() == section.id())
                    .and_then(|(_, h)| h.timestamp());
                let parent_volume = parent_entry.parent_volume();
                let parent_volume = (
                    parent_volume.title().to_owned(),
                    parent_entry.parent_volume_part(),
                    parent_volume.parts_count(),
                );

                home_html::RecentSection {
                    id: section.id(),
                    parent_volume,
                    parent_entry: parent_entry.title().to_owned(),
                    in_entry: (in_entry + 1, parent_entry.section_ids().len()),
                    date: section.date().format("%Y-%m-%d").to_string(),
                    previous: previous.map(|s| (s.id(), s.description().to_owned())),
                    description: section.description().to_owned(),
                    summary: section.summary().to_owned(),
                    length: section.length_string(),
                    read,
                }
            })
            .collect();
        let expand = user.preferences().get("expand_recents");
        let expand = match expand {
            Some(expand) => expand == "true",
            None => true,
        };
        home_html::RecentWidget {
            sections: recents,
            expand,
        }
    };

    let library_widget = || {
        let volumes = index
            .volumes()
            .filter(|v| v.kind() == volume::Kind::Journal)
            .map(|v| home_html::LibraryVolume {
                title: v.title().to_owned(),
                id: v.id().to_owned(),
                subtitle: v.subtitle().map(|s| s.to_owned()),
                entry_count: v.entry_ids().len(),
            })
            .collect();

        home_html::LibraryWidget {
            volumes,
            title: "The library".to_owned(),
        }
    };

    let extras_widget = || {
        let volumes = index
            .volumes()
            .filter(|v| v.kind() != volume::Kind::Journal && v.kind() != volume::Kind::Featured)
            .map(|v| home_html::LibraryVolume {
                title: v.title().to_owned(),
                id: v.id().to_owned(),
                subtitle: v.subtitle().map(|s| s.to_owned()),
                entry_count: v.entry_ids().len(),
            })
            .collect();

        home_html::LibraryWidget {
            volumes,
            title: "Extras".to_owned(),
        }
    };

    let last_widget = || {
        let section = user
            .history()
            .iter()
            .filter_map(|(s, h)| h.progress().map(|p| (s, p)))
            .next()
            .map(|(s, p)| {
                let entry = s.parent_entry();
                let index = s.index_in_parent();
                home_html::LastSection {
                    entry: entry.title().to_owned(),
                    summary: s.summary().to_owned(),
                    timestamp: p.1,
                    id: s.id(),
                    index: (index, entry.section_count()),
                    progress: (p.0, s.lines()),
                }
            });

        home_html::LastWidget { section }
    };

    let random_widget = || {
        // TODO make random.

        let wrap_entry = |entry: &Entry| {
            let volume = entry.parent_volume();
            let volume_part = (volume.parts_count() > 1).then_some(entry.parent_volume_part());

            home_html::RandomEntry {
                id: entry.id().to_owned(),
                summary: entry.summary().to_owned(),
                title: entry.title().to_owned(),
                volume: volume.title().to_owned(),
                volume_part,
            }
        };

        let unstarted_entries: Vec<_> = index
            .entries()
            .filter(|e| matches!(user.entry_progress(&e), EntryProgress::Unstarted))
            .collect();
        if !unstarted_entries.is_empty() {
            let entry =
                &unstarted_entries[rand::thread_rng().gen_range(0..unstarted_entries.len())];
            return home_html::RandomWidget::Unstarted(wrap_entry(entry));
        }

        let unfinished_entries: Vec<_> = index
            .entries()
            .filter_map(|e| {
                if let EntryProgress::InSection {
                    section_id,
                    section_index,
                    progress,
                    ..
                } = user.entry_progress(&e)
                {
                    Some((e, section_id, section_index, progress))
                } else {
                    None
                }
            })
            .collect();
        if let Some(entry) = unfinished_entries.last() {
            return home_html::RandomWidget::Unfinished {
                entry: wrap_entry(&entry.0),
                section_id: entry.1,
                section_index: entry.2,
                progress: entry.3,
            };
        }

        let mut read_again: Vec<_> = index
            .entries()
            .filter_map(|e| {
                if let EntryProgress::Finished { last_read } = user.entry_progress(&e) {
                    Some((e, last_read))
                } else {
                    None
                }
            })
            .collect();
        read_again.sort_by_key(|e| e.1);
        let entry = &read_again[rand::thread_rng().gen_range(0..read_again.len().min(10))];

        home_html::RandomWidget::ReadAgain {
            entry: wrap_entry(&entry.0),
            last_read: entry.1,
        }
    };

    for widget in user.widgets() {
        let widget: Box<dyn Widget> = match widget.as_ref() {
            "recent-widget" => Box::new(recent_widget()),
            "library-widget" => Box::new(library_widget()),
            "extras-widget" => Box::new(extras_widget()),
            "last-widget" => Box::new(last_widget()),
            "random-widget" => Box::new(random_widget()),
            _ => continue,
        };
        widgets.push(widget);
    }

    let intro = fs::read_to_string("content/edat.intro").unwrap();
    let intro_lines: Vec<&str> = intro.lines().filter(|l| l.len() > 0).collect();
    Ok(home_html::home(&headers, widgets, intro_lines))
}
