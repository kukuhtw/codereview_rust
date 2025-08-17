// src/models.rs
/*
=============================================================================
Project : AI CodeReview Rust
Author : Kukuh Tripamungkas Wicaksono (Kukuh TW)
Email : kukuhtw@gmail.com
WhatsApp : https://wa.me/628129893706
LinkedIn : https://id.linkedin.com/in/kukuhtw
=============================================================================/

*/
use askama::Template;
use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AppRow {
    pub id: i64,
    pub nama_aplikasi: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Template)]
#[template(path="index.html")]
pub struct IndexPage<'a> {
    pub apps: &'a [AppRow],
}

#[derive(Template)]
#[template(path="upload.html")]
pub struct UploadPage;

#[derive(Template)]
#[template(path="detail.html")]
pub struct DetailPage<'a> {
    pub app: &'a AppRow,
    pub files: &'a [FileRow],
}

#[derive(Template)]
#[template(path="analysis.html")]
pub struct AnalysisPage<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub back_href: &'a str,
    pub force_href: Option<&'a str>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FileRow {
    pub id: i64,
    pub app_id: i64,
    pub nama_file: String,
    pub nama_folder: Option<String>,
    pub full_path: String,
    pub line_count: Option<i32>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Application {
    pub id: i64,
    pub nama_aplikasi: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FileEntry {
    pub id: i64,
    pub app_id: i64,
    pub nama_file: String,
    pub nama_folder: Option<String>,
    pub full_path: String,
}

// ===== Tambahan untuk halaman “Semua Analisa” =====
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AnalysisJoinRow {
    pub file_id: i64,
    pub nama_file: String,
    pub full_path: String,
    pub analisa_fungsi: Option<String>,
    pub analisa_relasi_file: Option<String>,
    pub analisa_relasi_db: Option<String>,
}

#[derive(Template)]
#[template(path="analysis_all.html")]
pub struct AnalysisAllPage<'a> {
    pub app: &'a AppRow,
    pub rows: &'a [AnalysisJoinRow],
}
