// src/models.rs
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
    pub json: String,
}

#[derive(Template)]
#[template(path="upload.html")]
pub struct UploadPage;

#[derive(Debug, Serialize)]
pub struct FileWithAnalyses {
    pub id: i64,
    pub app_id: i64,
    pub nama_file: String,
    pub nama_folder: Option<String>,
    pub full_path: String,
    pub line_count: Option<i32>,
    pub fungsi_preview: Option<String>,
    pub relasi_file_preview: Option<String>,
    pub relasi_db_preview: Option<String>,
    pub has_graph: bool,
     pub row_no: Option<i64>,
}

pub struct Pagination {
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
    pub total_items: i64,
    pub first: usize,
    pub last: usize,
    pub prev: Option<usize>,
    pub next: Option<usize>,
    pub p_minus2: Option<usize>,
    pub p_minus1: Option<usize>,
    pub p_plus1: Option<usize>,
    pub p_plus2: Option<usize>,
    pub from: i64, // index awal di halaman ini (1-based)
    pub to: i64,   // index akhir di halaman ini
}

#[derive(Template)]
#[template(path="detail.html")]
pub struct DetailPage<'a> {
    pub app: &'a AppRow,
    pub files: &'a [FileWithAnalyses],
    pub pagination: Pagination, // ⟵ BARU
    pub search: Option<String>,
}

#[derive(Template)]
#[template(path="analysis.html")]
pub struct AnalysisPage <'a> {
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

#[derive(Template)]
#[template(path="graph.html")]
pub struct GraphPage<'a> {
    pub app: &'a AppRow,
    pub file_name: &'a str,
    pub graph_js: &'a str,
}
