use serde::Deserialize;

use super::*;

pub async fn entry(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath(entry): ReqPath<String>,
) -> Result<Response, Markup> {
    use html::pages::entry as entry_html;

    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    match index.entry(entry.clone()) {
        Ok(entry) => Ok(no_cache(entry_html::entry(
            &headers,
            &entry,
            &user,
            entry_html::EntryDestination::Top,
        ))),
        Err(_) => Err(html::pages::entry::error(&headers, &entry)),
    }
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
    let user = auth::get_user(&headers, &index)?;

    match index.section(section) {
        Ok(section) => {
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
        Err(_) => Err(html::pages::entry::section_error(&headers, section)),
    }
}

pub async fn forum(headers: HeaderMap, State(state): State<AppState>) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    Ok(no_cache(html::pages::forum::forum(&headers, &user)))
}

pub async fn history(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    Ok(no_cache(html::pages::history::history(&headers, &user)))
}

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    Ok(no_cache(html::pages::home::home(&headers, &user)))
}

pub async fn profile(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    Ok(no_cache(html::pages::profile::profile(&headers, &user)))
}

pub async fn search(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath(searches): ReqPath<String>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let _ = auth::get_user(&headers, &index)?;

    let searches: Vec<_> = searches.split(",").filter(|s| !s.is_empty()).collect();

    Ok(no_cache(html::pages::search::search(&headers, &index, &searches)))
}

pub async fn search_empty(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    let _ = auth::get_user(&headers, &index)?;

    Ok(no_cache(html::pages::search::search(&headers, &index, &[])))
}

pub async fn terminal(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;
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
    let user = auth::get_user(&headers, &index)?;

    match index.volume(volume.clone()) {
        Ok(volume) => Ok(no_cache(html::pages::volume::volume(
            &headers, &volume, &user,
        ))),
        Err(_) => Err(html::pages::volume::error(&headers, &volume)),
    }
}

pub async fn volumes(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, Markup> {
    let index = state.index.lock().unwrap();
    auth::get_user(&headers, &index)?;
    Ok(no_cache(html::pages::volume::library(&headers, &index)))
}
