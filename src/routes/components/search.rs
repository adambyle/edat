use super::*;

pub async fn entry(
    ReqPath((id, query)): ReqPath<(String, String)>,
    State(state): State<AppState>,
) -> Markup {
    let index = state.index.lock().await;
    let searches: Vec<_> = query.split(",").filter(|s| !s.is_empty()).collect();

    html::components::search::entry(&index, &id, &searches)
}

pub async fn intro(
    ReqPath((id, query)): ReqPath<(String, String)>,
    State(state): State<AppState>,
) -> Markup {
    let index = state.index.lock().await;
    let searches: Vec<_> = query.split(",").filter(|s| !s.is_empty()).collect();

    html::components::search::intro(&index, &id, &searches)
}

pub async fn section(
    ReqPath((id, query)): ReqPath<(u32, String)>,
    State(state): State<AppState>,
) -> Markup {
    let index = state.index.lock().await;
    let searches: Vec<_> = query.split(",").filter(|s| !s.is_empty()).collect();

    html::components::search::section(&index, id, &searches)
}

pub async fn volume(
    ReqPath((id, query)): ReqPath<(String, String)>,
    State(state): State<AppState>,
) -> Markup {
    let index = state.index.lock().await;
    let searches: Vec<_> = query.split(",").filter(|s| !s.is_empty()).collect();

    html::components::search::volume(&index, &id, &searches)
}
