use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use axum::{
    routing::{get, post},
    Router,
};

mod data;
mod html;
mod routes;
mod search;

#[derive(Clone)]
pub struct AppState {
    index: Arc<Mutex<data::Index>>,
}

#[tokio::main]
async fn main() {
    let index_load_start = Instant::now();
    let index = data::Index::init();
    let index_load_elapsed = index_load_start.elapsed();
    println!("Index loaded in {index_load_elapsed:?}");

    let state = AppState {
        index: Arc::new(Mutex::new(index)),
    };

    let app = Router::new()
        .route("/", get(routes::pages::home::home))
        .route("/archive", get(routes::files::archive))
        .route("/cmd", post(routes::cmd::cmd))
        .route("/image/:file", get(routes::files::image))
        .route("/image/:file", post(routes::cmd::image_upload))
        .route("/login/:name/:code", post(routes::auth::login))
        .route("/preferences", post(routes::user::set_preferences))
        .route("/profile", get(routes::pages::profile))
        .route("/read/:id", post(routes::user::read))
        .route("/register", post(routes::user::register))
        .route("/script/:file", get(routes::files::script))
        .route("/style/:file", get(routes::files::style))
        .route("/terminal", get(routes::pages::terminal))
        .route("/widgets", post(routes::user::set_widgets))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
