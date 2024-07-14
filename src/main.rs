use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use axum::{
    routing::{delete, get, post},
    Router,
};
use tokio::net::TcpListener;

mod data;
mod html;
mod image;
mod routes;
mod search;

#[derive(Clone)]
pub struct AppState {
    index: Arc<Mutex<data::Index>>,
}

#[tokio::main]
async fn main() {
    image::entry_image(
        "Breaching the Gates of Heaven",
        "After forming unbreakable memories with Bronwyn and seeing favorable odds, I am forced to fix a breaking friendship when feelings aren’t returned.",
    );

    let index_load_start = Instant::now();
    let mut index = data::Index::init();
    if let Some(arg) = std::env::args().nth(1) {
        if arg == "reindex" {
            index.save_all();
        }
    }
    let index_load_elapsed = index_load_start.elapsed();
    println!("Index loaded in {index_load_elapsed:?}");

    let state = AppState {
        index: Arc::new(Mutex::new(index)),
    };

    let app = Router::new()
        .route("/", get(routes::pages::home))
        .route("/archive", get(routes::files::archive))
        .route("/asset/:file", get(routes::files::asset))
        .route("/cmd", post(routes::cmd::cmd))
        .route(
            "/components/library-search/:query",
            get(routes::components::library_search),
        )
        .route("/comment/:section/:line", post(routes::post::comment))
        .route(
            "/edit_comment/:section/:uuid",
            post(routes::post::edit_comment),
        )
        .route("/entry/:entry", get(routes::pages::entry))
        .route("/forum", get(routes::pages::forum))
        .route("/history", get(routes::pages::history))
        .route("/image/:file", get(routes::files::image))
        .route("/image/:file", post(routes::cmd::image_upload))
        .route("/library", get(routes::pages::volumes))
        .route("/login/:name/:code", post(routes::auth::login))
        .route("/preferences", post(routes::user::set_preferences))
        .route("/preview", get(routes::files::preview))
        .route("/profile", get(routes::pages::profile))
        .route("/read/:id", post(routes::user::read))
        .route("/register", post(routes::user::register))
        .route(
            "/remove_comment/:section/:uuid",
            delete(routes::delete::comment),
        )
        .route("/script/:file", get(routes::files::script))
        .route("/search", get(routes::pages::search_empty))
        .route("/search/:query", get(routes::pages::search))
        .route(
            "/search/entry/:id/:query",
            get(routes::components::search::entry),
        )
        .route(
            "/search/intro/:id/:query",
            get(routes::components::search::intro),
        )
        .route(
            "/search/section/:id/:query",
            get(routes::components::search::section),
        )
        .route(
            "/search/volume/:id/:query",
            get(routes::components::search::volume),
        )
        .route("/section/:id", get(routes::pages::entry_by_section))
        .route("/style/:file", get(routes::files::style))
        .route("/terminal", get(routes::pages::terminal))
        .route("/thread/:section/:line", get(routes::components::thread))
        .route(
            "/unremove_comment/:section/:uuid",
            post(routes::post::unremove_comment),
        )
        .route("/volume/:volume", get(routes::pages::volume))
        .route("/widgets", post(routes::user::set_widgets))
        .with_state(state);

    let listener = listener().await;
    axum::serve(listener, app).await.unwrap();
}

#[cfg(debug_assertions)]
async fn listener() -> TcpListener {
    TcpListener::bind("0.0.0.0:3000").await.unwrap()
}

#[cfg(not(debug_assertions))]
async fn listener() -> TcpListener {
    TcpListener::bind("0.0.0.0:80").await.unwrap()
}
