use super::*;

pub mod search;

pub async fn library_search(
    ReqPath(query): ReqPath<String>,
    State(state): State<AppState>,
) -> Markup {
    let index = state.index.lock().unwrap();
    let words: Vec<_> = query.split(",").collect();
    html::components::library_search(&index, &words)
}

pub async fn thread(
    headers: HeaderMap,
    ReqPath((section, line)): ReqPath<(u32, usize)>,
    State(state): State<AppState>,
) -> Markup {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index, None, false).unwrap();
    html::components::thread(&user, section, line)
}
