use super::*;

pub async fn library_search(
    ReqPath(query): ReqPath<String>,
    State(state): State<AppState>,
) -> Markup {
    let index = state.index.lock().unwrap();
    let words: Vec<_> = query.split(",").collect();
    html::components::library_search(&index, &words)
}
