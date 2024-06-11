use std::fs;

use html::pages::home as home_html;
use rand::Rng;

use super::*;

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {

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
        sections.sort_by_key(|s| (s.date(), s.id()));
        sections.reverse();

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
                    .map(|(_, h)| h.timestamp());
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
            .filter(|e| matches!(user.entry_progress(&e), None))
            .collect();
        if !unstarted_entries.is_empty() {
            let entry =
                &unstarted_entries[rand::thread_rng().gen_range(0..unstarted_entries.len())];
            return home_html::RandomWidget::Unstarted(wrap_entry(entry));
        }

        let unfinished_entries: Vec<_> = index
            .entries()
            .filter_map(|e| {
                if let Some(EntryProgress::InSection {
                    section_id,
                    section_index,
                    line,
                    ..
                }) = user.entry_progress(&e)
                {
                    Some((e, section_id, section_index, line))
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
                if let Some(EntryProgress::Finished { last_read }) = user.entry_progress(&e) {
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
        let widget: Box<dyn home_html::Widget> = match widget.as_ref() {
            "recent-widget" => Box::new(recent_widget()),
            "library-widget" => Box::new(library_widget()),
            "extras-widget" => Box::new(extras_widget()),
            "last-widget" => Box::new(last_widget(&user)),
            "random-widget" => Box::new(random_widget()),
            _ => continue,
        };
        widgets.push(widget);
    }

    let intro = fs::read_to_string("content/edat.intro").unwrap();
    let intro_lines: Vec<&str> = intro.lines().filter(|l| l.len() > 0).collect();
    Ok(home_html::home(&headers, widgets, intro_lines))
}

fn last_widget<'index>(user: &'index User) -> home_html::LastWidget<'index> {
    let section = user
        .history()
        .into_iter()
        .find(|(_, h)| !matches!(h, SectionProgress::Finished { .. }));

    home_html::LastWidget { section }
}
