// src/db.rs
/*
=============================================================================
Project : AI CodeReview Rust
Author : Kukuh Tripamungkas Wicaksono (Kukuh TW)
Email : kukuhtw@gmail.com
WhatsApp : https://wa.me/628129893706
LinkedIn : https://id.linkedin.com/in/kukuhtw
=============================================================================/

*/
use sqlx::{MySql, Pool};

pub async fn init_pool() -> Pool<MySql> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("DB connect failed")
}
