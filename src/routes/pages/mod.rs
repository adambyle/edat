use super::*;

pub mod home;

pub async fn profile(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    Ok(html::pages::profile::profile(&headers, &user))
}

pub async fn terminal(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;
    Ok(html::pages::terminal(
        &headers,
        user.privilege() == UserPrivilege::Owner,
    ))
}
