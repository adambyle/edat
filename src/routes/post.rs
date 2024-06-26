use super::*;

pub async fn comment(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath((section, line)): ReqPath<(u32, usize)>,
    body: String,
) -> StatusCode {
    let mut index = state.index.lock().unwrap();

    let Ok(user) = auth::get_user(&headers, &index) else {
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let author = user.id().to_owned();

    let Ok(mut section) = index.section_mut(section) else {
        return StatusCode::NOT_FOUND;
    };

    section.add_comment(author, line, &body);

    StatusCode::OK
}
