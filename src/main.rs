use axum::{routing::{get, post}, Router};

#[tokio::main]
async fn main() {
    let state = data::Index::init();

    let app = Router::new()
        .route("/script/:file", get(routes::script))
        .route("/style/:file", get(routes::style))
        .route("/login/:name/:code", post(routes::login))
        .route("/register", post(routes::register))
        .route("/", get(routes::home))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("192.168.1.154:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod routes;

mod data;

mod html;
