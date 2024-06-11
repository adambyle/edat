use std::{
    fs::{self, File},
    io::{self},
    path::{Path, PathBuf},
};

use chrono::{Datelike, Utc};
use zip::{write::SimpleFileOptions, ZipWriter};

use super::*;

pub async fn script(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file(format!("static/scripts/{}", file_name), "text/javascript")
}

pub async fn style(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file(format!("static/styles/{}", file_name), "text/css")
}

pub async fn image(ReqPath(file_name): ReqPath<String>) -> impl IntoResponse {
    static_file(format!("content/images/{}", file_name), "image/jpeg")
}

pub async fn archive() -> impl IntoResponse {
    let now = Utc::now();
    let archive_path = format!("edat-{}-{}-{}.zip", now.year(), now.month(), now.day());
    let archive_file = File::create(&archive_path).unwrap();
    let mut zip = ZipWriter::new(archive_file);

    fn directory(path: PathBuf, zip: &mut ZipWriter<File>) {
        let options = SimpleFileOptions::default();
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                directory(path, zip);
            } else {
                zip.start_file(path.to_str().unwrap(), options).unwrap();
                let mut file = File::open(path).unwrap();
                io::copy(&mut file, zip).unwrap();
            }
        }
    }

    directory("content".into(), &mut zip);
    directory("users".into(), &mut zip);
    directory("archived".into(), &mut zip);
    zip.finish().unwrap();

    let response = static_file(archive_path.clone(), "application/zip");
    fs::remove_file(archive_path).unwrap();
    response
}

fn static_file(path: String, content_type: &'static str) -> Response {
    let path = Path::new(&path);
    match fs::read_to_string(&path) {
        Ok(content) => (
            [
                (header::CONTENT_TYPE, content_type),
                (
                    header::CONTENT_DISPOSITION,
                    &format!(
                        "inline; filename=\"{}\"",
                        path.file_name().unwrap().to_string_lossy()
                    ),
                ),
            ],
            content,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
