extern crate dotenv;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::{env, sync::Arc};
use tokio_postgres::NoTls;

mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let conn = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let (client, connection) = tokio_postgres::connect(conn.as_str(), NoTls).await.unwrap();
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
