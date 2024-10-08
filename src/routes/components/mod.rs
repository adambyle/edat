use super::*;

pub mod search;

pub async fn library_search(
    ReqPath(query): ReqPath<String>,
    State(state): State<AppState>,
) -> Markup {
    let index = state.index.lock().await;
    let words: Vec<_> = query.split(",").collect();
    html::components::library_search(&index, &words)
}

pub async fn thread(
    headers: HeaderMap,
    ReqPath((section, line)): ReqPath<(u32, usize)>,
    State(state): State<AppState>,
) -> Result<Markup, Markup> {
    let index = state.index.lock().await;
    let user = auth::get_user(&headers, &index, None, false)?;
    Ok(html::components::thread(&user, section, line))
}
