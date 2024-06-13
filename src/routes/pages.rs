use super::*;

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    Ok(html::pages::home::home(&headers, &user))
}

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

pub async fn volume(
    headers: HeaderMap,
    State(state): State<AppState>,
    ReqPath(volume): ReqPath<String>,
) -> Result<Markup, Markup> {
    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    match index.volume(volume.clone()) {
        Ok(volume) => Ok(html::pages::volume::volume(&headers, &volume, &user)),
        Err(_) => Err(html::pages::volume::error(&headers, &volume)),
    }
}

pub async fn volumes(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {
    let index = state.index.lock().unwrap();
    auth::get_user(&headers, &index)?;
    Ok(html::pages::volume::library(&headers, &index))
}
