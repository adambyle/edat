use serde::Deserialize;

use crate::image;

use super::*;

pub async fn entry(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath(entry): ReqPath<String>,
) -> Result<Response, Markup> {
    use html::pages::entry as entry_html;

    let index = state.index.lock().unwrap();

    let entry = match index.entry(entry.clone()) {
        Ok(entry) => entry,
        Err(_) => return Err(html::pages::entry::error(&headers, &entry)),
    };

    image::entry_image(entry.title(), entry.summary());

    let user = auth::get_user(&headers, &index, Some(entry.title().to_owned()), true)?;

    Ok(no_cache(entry_html::entry(
        &headers,
        &entry,
        &user,
        entry_html::EntryDestination::Top,
    )))
}

#[derive(Deserialize)]
pub struct EntryBySectionBody {
    line: Option<usize>,
}

pub async fn entry_by_section(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath(section): ReqPath<u32>,
    Query(options): Query<EntryBySectionBody>,
) -> Result<Response, Markup> {
    use html::pages::entry as entry_html;

    let index = state.index.lock().unwrap();

    let section = match index.section(section) {
        Ok(section) => section,
        Err(_) => return Err(html::pages::entry::section_error(&headers, section)),
    };

    let section_index = if section.parent_entry().section_count() == 1 {
        "Standalone".to_owned()
    } else {
        format!("Section {}", section.index_in_parent() + 1)
    };

    image::section_image(
        section.parent_entry().title(),
        section.summary(),
        &section_index,
        &crate::data::date_string(&section.date()),
    );

    let user = auth::get_user(
        &headers,
        &index,
        Some(section.parent_entry().title().to_owned()),
        true,
    )?;

    let destination = if let Some(line) = options.line {
        entry_html::EntryDestination::Line(section.id(), line)
    } else {
        entry_html::EntryDestination::Section(section.id())
    };

    Ok(no_cache(entry_html::entry(
        &headers,
        &section.parent_entry(),
        &user,
        destination,
    )))
}

pub async fn music(headers: HeaderMap, State(state): State<AppState>) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();

    Ok(no_cache(html::pages::music::music(&headers)))
}

pub async fn history(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index, Some("Upload history".to_owned()), false)?;

    Ok(no_cache(html::pages::history::history(&headers, &user)))
}

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index, Some("Home".to_owned()), false)?;

    Ok(no_cache(html::pages::home::home(&headers, &user)))
}

pub async fn profile(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index, Some("Profile".to_owned()), false)?;

    Ok(no_cache(html::pages::profile::profile(&headers, &user)))
}

pub async fn search(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath(searches): ReqPath<String>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let _ = auth::get_user(&headers, &index, Some("Search".to_owned()), false)?;

    let searches: Vec<_> = searches.split(",").filter(|s| !s.is_empty()).collect();

    Ok(no_cache(html::pages::search::search(
        &headers, &index, &searches,
    )))
}

pub async fn search_empty(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let _ = auth::get_user(&headers, &index, None, false)?;

    Ok(no_cache(html::pages::search::search(&headers, &index, &[])))
}

pub async fn terminal(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index, Some("Terminal".to_owned()), false)?;
    Ok(html::pages::terminal(
        &headers,
        user.privilege() == UserPrivilege::Owner,
    ))
}

pub async fn volume(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath(volume): ReqPath<String>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();

    let volume = match index.volume(volume.clone()) {
        Ok(volume) => volume,
        Err(_) => return Err(html::pages::volume::error(&headers, &volume)),
    };

    let user = auth::get_user(&headers, &index, Some(volume.title().to_owned()), false)?;

    Ok(no_cache(html::pages::volume::volume(
        &headers, &volume, &user,
    )))
}

pub async fn volumes(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    auth::get_user(&headers, &index, Some("The library".to_owned()), false)?;
    Ok(no_cache(html::pages::volume::library(&headers, &index)))
}
