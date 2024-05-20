use axum::{routing::{get, post}, Router};
use data::load_users;

#[derive(Clone)]
struct AppState {
    users: Vec<data::User>,
}

#[tokio::main]
async fn main() {
    let users = load_users();
    let state = AppState {
        users,
    };

    let app = Router::new()
        .route("/script/:file", get(routes::script))
        .route("/style/:file", get(routes::style))
        .route("/login/:name/:code", post(routes::login))
        .route("/", get(routes::home))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod routes;

mod data;

mod html;
