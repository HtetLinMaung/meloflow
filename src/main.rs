use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use tokio_postgres::NoTls;

mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = "host=150.95.82.125 port=5432 user=postgres password=P@ssword dbname=meloflow_db";
    let (client, connection) = tokio_postgres::connect(conn, NoTls).await.unwrap();
    let client = Arc::new(client);

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(handlers::stream_song)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
