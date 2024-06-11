// TODO remove this use.
use html::pages::home as home_html;

use super::*;

pub async fn home(headers: HeaderMap, State(state): State<AppState>) -> Result<Markup, Markup> {

    let index = state.index.lock().unwrap();
    let user = auth::get_user(&headers, &index)?;

    Ok(home_html::home(&headers, &user))
}
