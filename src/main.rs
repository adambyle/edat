use std::sync::{Arc, Mutex};

use axum::{routing::{get, post}, Router};

#[derive(Clone)]
pub struct AppState {
    index: Arc<Mutex<data::Index>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        index: Arc::new(Mutex::new(data::Index::init())),
    };

    let app = Router::new()
        .route("/script/:file", get(routes::script))
        .route("/style/:file", get(routes::style))
        .route("/image/:file", get(routes::image))
        .route("/image/:file", post(routes::image_upload))
        .route("/login/:name/:code", post(routes::login))
        .route("/register", post(routes::register))
        .route("/preferences", post(routes::preferences))
        .route("/terminal", get(routes::terminal))
        .route("/cmd", post(routes::cmd))
        .route("/archive", get(routes::archive))
        .route("/", get(routes::home))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod routes;

mod data;

mod html;
