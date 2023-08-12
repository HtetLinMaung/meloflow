use actix_web::Error;
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::{fs::File, sync::Arc};

use tokio_postgres::Client;

async fn fetch_song_path_from_db(
    song_id: i32,
    client: web::Data<Arc<Client>>,
) -> Result<String, tokio_postgres::Error> {
    let row = client
        .query_one(
            "select file_path from songs where song_id = $1",
            &[&song_id],
        )
        .await?;
    let file_path: String = row.get(0);

    Ok(file_path)
}

#[get("/songs/{song_id}")]
async fn stream_song(
    song_id: web::Path<i32>,
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
) -> Result<HttpResponse, Error> {
    let file_path = match fetch_song_path_from_db(*song_id, client).await {
        Ok(path) => path,
        Err(_) => {
            return Err(actix_web::error::ErrorNotFound(
                "Song not found in database",
            ))
        }
    };

    let path: PathBuf = file_path.into();
    if !path.exists() {
        return Err(actix_web::error::ErrorNotFound("File not found"));
    }

    let mut file = File::open(&path).unwrap();

    let file_size = file.metadata().unwrap().len();

    if let Some(range) = req.headers().get("Range") {
        let (start, end) = parse_range(&range.to_str().unwrap(), file_size).unwrap();
        let mut file_chunk = vec![0; (end - start + 1) as usize];
        file.seek(SeekFrom::Start(start)).unwrap();
        file.read_exact(&mut file_chunk).unwrap();

        Ok(HttpResponse::PartialContent()
            .append_header((
                "Content-Range",
                format!("bytes {}-{}/{}", start, end, file_size),
            ))
            .append_header(("Content-Type", "audio/mpeg"))
            .body(file_chunk))
    } else {
        let mut entire_file = Vec::with_capacity(file_size as usize);
        file.read_to_end(&mut entire_file).unwrap();
        Ok(HttpResponse::Ok().body(entire_file))
    }
}

fn parse_range(range: &str, file_size: u64) -> Option<(u64, u64)> {
    if !range.starts_with("bytes=") {
        return None;
    }

    let ranges: Vec<&str> = range.trim_start_matches("bytes=").split('-').collect();
    let start: u64 = ranges[0].parse().ok()?;
    let end: u64 = ranges
        .get(1)
        .and_then(|&s| s.parse().ok())
        .unwrap_or(file_size - 1);

    Some((start, end))
}
