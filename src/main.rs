use axum::{routing::{get, post}, Router};
use data::{load_content, load_users};

#[derive(Clone)]
struct AppState {
    users: Vec<data::User>,
    content: data::Content,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        users: load_users(),
        content: load_content(),
    };

    let app = Router::new()
        .route("/script/:file", get(routes::script))
        .route("/style/:file", get(routes::style))
        .route("/login/:name/:code", post(routes::login))
        .route("/record/:mode", post(routes::record))
        .route("/", get(routes::home))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("192.168.1.154:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod routes;

mod data;

mod html;
