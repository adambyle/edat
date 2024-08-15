use super::*;

pub async fn comment(
    State(state): State<AppState>,
    ReqPath((section, uuid)): ReqPath<(u32, u128)>,
) -> StatusCode {
    let mut index = state.index.lock().await;
    
    let Ok(mut section) = index.section_mut(section) else {
        return StatusCode::NOT_FOUND;
    };

    section.remove_comment(uuid);

    StatusCode::OK
}
