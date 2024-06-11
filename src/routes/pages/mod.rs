use chrono::Utc;

use super::*;

pub mod home;

pub async fn profile(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    // Get the sections read in the last two months.
    let two_months_ago = Utc::now().timestamp() - 60 * 60 * 24 * 60;
    let sections = user
        .history()
        .iter()
        .filter_map(|(s, p)| {
            let Some((progress, timestamp)) = p.progress() else {
                return None;
            };
            if timestamp < two_months_ago {
                return None;
            }

            Some(html::pages::profile::ViewedSection {
                description: s.description().to_owned(),
                timestamp: timestamp,
                entry: s.parent_entry().title().to_owned(),
                id: s.id(),
                index: (s.index_in_parent(), s.parent_entry().section_count()),
                progress: (progress, s.lines()),
            })
        })
        .collect();

    let profile_data = html::pages::profile::ProfileData {
        widgets: user.widgets().to_owned(),
        sections,
    };
    Ok(html::pages::profile::profile(&headers, profile_data))
}

pub async fn terminal(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;
    Ok(html::pages::terminal(
        &headers,
        user.privilege() == UserPrivilege::Owner,
    ))
}
