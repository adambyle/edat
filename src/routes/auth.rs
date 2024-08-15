use super::*;

/// Attempt to login a user with the given credentials.
pub async fn login(
    State(state): State<AppState>,
    ReqPath((name, code)): ReqPath<(String, String)>,
) -> Response {
    let index = state.index.lock().await;
    let name = name.to_lowercase().replace(char::is_whitespace, "");
    let code = code.to_lowercase();

    // Find a user whose first name matches the input or whose id matches the input.
    for user in index.users() {
        if (name == user.first_name().to_lowercase() || &name == user.id()) && user.has_code(&code)
        {
            return (StatusCode::OK, user.id().to_owned()).into_response();
        }
    }

    StatusCode::UNAUTHORIZED.into_response()
}

/// Get the user information from the request.
/// 
/// Returns an error wrapping either the login page or the setup page
/// if either is needed.
pub(super) fn get_user<'index>(
    headers: &HeaderMap,
    index: &'index Index,
    title: Option<String>,
    show_panel: bool,
) -> Result<User<'index>, maud::Markup> {
    // Get the user.
    let user = get_cookie(headers, "edat_user").and_then(|u| index.user(u.to_owned()).ok());
    
    let Some(user) = user else {
        return Err(html::pages::login(headers, title, show_panel));
    };

    if !user.is_init() {
        return Err(html::pages::setup(headers, index));
    }

    Ok(user)
}
