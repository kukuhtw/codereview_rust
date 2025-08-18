// src/handlers.rs

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
use futures_util::TryStreamExt;
use warp::{Reply, Rejection};
use warp::multipart::FormData;
use warp::Buf;
use sqlx::MySqlPool;
use std::io::Write;
use warp::http::StatusCode;
use serde_json::json;
use chrono::Utc;

use crate::models::{
    IndexPage, UploadPage, DetailPage, AnalysisPage, AnalysisAllPage,
    AppRow, AnalysisJoinRow, FileWithAnalyses,
};

fn truncate_words(s: &str, max_words: usize) -> String {
    let mut out = String::new();
    let mut count = 0usize;
    for w in s.split_whitespace() {
        if count >= max_words { break; }
        if !out.is_empty() { out.push(' '); }
        out.push_str(w);
        count += 1;
    }
    if s.split_whitespace().count() > max_words {
        out.push_str(" …");
    }
    out
}

pub async fn upload_page() -> Result<impl Reply, Rejection> {
    let page = UploadPage;
    Ok(warp::reply::html(page.render().unwrap()))
}

pub async fn upload_zip(form: FormData, pool: MySqlPool) -> Result<impl Reply, Rejection> {
    let mut app_name = "MyApp".to_string();
    let mut zip_path: Option<String> = None;

    let mut parts = form;
    while let Some(part) = parts.try_next().await.map_err(|_| warp::reject())? {
        match part.name() {
            "app_name" => {
                let mut data = Vec::new();
                let mut s = part.stream();
                while let Some(mut chunk) = s.try_next().await.map_err(|_| warp::reject())? {
                    data.extend_from_slice(chunk.chunk());
                    chunk.advance(chunk.remaining());
                }
                app_name = String::from_utf8(data).unwrap_or_else(|_| "MyApp".to_string());
            }
            "file" => {
                let mut fname = std::env::temp_dir();
                fname.push(format!("{}.zip", uuid::Uuid::new_v4()));
                let mut f = std::fs::File::create(&fname).map_err(|_| warp::reject())?;

                let mut s = part.stream();
                while let Some(mut chunk) = s.try_next().await.map_err(|_| warp::reject())? {
                    f.write_all(chunk.chunk()).map_err(|_| warp::reject())?;
                    chunk.advance(chunk.remaining());
                }
                zip_path = Some(fname.to_string_lossy().to_string());
            }
            _ => {}
        }
    }

    let zip_path = zip_path.ok_or_else(warp::reject)?;
    let app_id = crate::services::extract_and_store(&pool, &app_name, &zip_path)
        .await
        .map_err(|_| warp::reject())?;

    let res = warp::http::Response::builder()
        .status(302)
        .header("Location", format!("/apps/{app_id}"))
        .body("Uploaded")
        .unwrap();
    Ok(res)
}

pub async fn list_apps(pool: MySqlPool) -> Result<impl warp::Reply, warp::Rejection> {
    match sqlx::query_as::<_, AppRow>(
        "SELECT id, nama_aplikasi, created_at FROM applications ORDER BY id DESC"
    ).fetch_all(&pool).await {
        Ok(rows) => {
            let page = IndexPage { apps: &rows };
            Ok(warp::reply::html(page.render().unwrap()))
        }
        Err(e) => {
            let html = format!("<h3>DB error</h3><pre>{e}</pre>\
                                <p>Cek DATABASE_URL / DB & tabel.</p>");
            Ok(warp::reply::html(html))
        }
    }
}

pub async fn app_detail(app_id: i32, pool: MySqlPool) -> Result<impl Reply, Rejection> {
    let app: Option<AppRow> = sqlx::query_as(
        "SELECT id, nama_aplikasi, created_at FROM applications WHERE id=?",
    )
    .bind(app_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| warp::reject())?;

    let Some(app) = app else {
        let html = format!("Aplikasi dengan id {} tidak ditemukan.", app_id);
        return Ok(warp::reply::with_status(
            warp::reply::html(html),
            StatusCode::NOT_FOUND,
        ));
    };

    // files + metadata + analysis + json_graph
    let rows = sqlx::query!(
        r#"
        SELECT
            f.id, f.app_id, f.nama_file, f.nama_folder, f.full_path, f.json_graph,
            m.line_count,
            a.analisa_fungsi, a.analisa_relasi_file, a.analisa_relasi_db
        FROM files f
        LEFT JOIN file_metadata m ON m.file_id = f.id
        LEFT JOIN analysis a ON a.file_id = f.id
        WHERE f.app_id = ?
        ORDER BY f.id
        "#,
        app_id
    ).fetch_all(&pool).await.map_err(|_| warp::reject())?;

    let mut files: Vec<FileWithAnalyses> = Vec::with_capacity(rows.len());
    for r in rows {
        let fungsi_preview = r.analisa_fungsi.as_deref().map(|s| truncate_words(s, 50));
        let relasi_file_preview = r.analisa_relasi_file.as_deref().map(|s| truncate_words(s, 50));
        let relasi_db_preview = r.analisa_relasi_db.as_deref().map(|s| truncate_words(s, 50));

        files.push(FileWithAnalyses {
            id: r.id as i64,
            app_id: r.app_id as i64,
            nama_file: r.nama_file,
            nama_folder: r.nama_folder,
            full_path: r.full_path,
            line_count: r.line_count,
            fungsi_preview,
            relasi_file_preview,
            relasi_db_preview,
            has_graph: r.json_graph.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false),
        });
    }

    let page = DetailPage { app: &app, files: &files };
    let html = page.render().map_err(|_| warp::reject())?;
    Ok(warp::reply::with_status(warp::reply::html(html), StatusCode::OK))
}

pub async fn app_summary(app_id: i32, pool: MySqlPool, force: bool) -> Result<impl Reply, Rejection> {
    if !force {
        if let Some(row) = sqlx::query!("SELECT summary FROM app_summary WHERE app_id=?", app_id)
            .fetch_optional(&pool).await.map_err(|_| warp::reject())?
        {
            if let Some(s) = row.summary {
                let page = AnalysisPage {
                    title: "Summary Aplikasi",
                    content: &s,
                    back_href: "/apps",
                    force_href: Some(&format!("/apps/{}/summary/force", app_id)),
                };
                return Ok(warp::reply::html(page.render().unwrap()));
            }
        }
    }

    let rows = sqlx::query!(
        r#"SELECT f.full_path, f.content_file, m.line_count, m.imports, m.sql_queries
           FROM files f LEFT JOIN file_metadata m ON m.file_id=f.id
           WHERE f.app_id=? ORDER BY f.id"#,
        app_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| warp::reject())?;

    let mut payload = String::new();
    const SNIPPET_CHARS: usize = 2000;
    for r in rows {
        let snippet = r
            .content_file
            .as_deref()
            .map(|s| {
                if s.len() > SNIPPET_CHARS {
                    format!("{}...\n[truncated]", &s[..SNIPPET_CHARS])
                } else {
                    s.to_string()
                }
            })
            .unwrap_or_default();

        payload.push_str(&format!(
            "- {} | lines={:?}\nimports:\n{}\nsql:\n{}\ncontent:\n{}\n\n",
            r.full_path,
            r.line_count,
            r.imports.unwrap_or_default(),
            r.sql_queries.unwrap_or_default(),
            snippet
        ));
    }

    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let summary =
        crate::openai::summary_app(&api_key, &payload).await.map_err(|_| warp::reject())?;

    sqlx::query(
        "INSERT INTO app_summary (app_id, summary) VALUES (?, ?)
         ON DUPLICATE KEY UPDATE summary=VALUES(summary), created_at=CURRENT_TIMESTAMP",
    )
    .bind(app_id)
    .bind(&summary)
    .execute(&pool)
    .await
    .map_err(|_| warp::reject())?;

    let page = AnalysisPage {
        title: "Summary Aplikasi",
        content: &summary,
        back_href: "/apps",
        force_href: Some(&format!("/apps/{}/summary/force", app_id)),
    };
    Ok(warp::reply::html(page.render().unwrap()))
}

pub async fn app_analysis_all(app_id: i32, pool: MySqlPool) -> Result<impl Reply, Rejection> {
    let app: Option<AppRow> = sqlx::query_as(
        "SELECT id, nama_aplikasi, created_at FROM applications WHERE id=?",
    )
    .bind(app_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| warp::reject())?;

    let Some(app) = app else {
        let html = format!("Aplikasi dengan id {} tidak ditemukan.", app_id);
        return Ok(warp::reply::with_status(
            warp::reply::html(html),
            StatusCode::NOT_FOUND,
        ));
    };

    let rows: Vec<AnalysisJoinRow> = sqlx::query_as(
        r#"
        SELECT
            f.id AS file_id,
            f.nama_file,
            f.full_path,
            a.analisa_fungsi,
            a.analisa_relasi_file,
            a.analisa_relasi_db
        FROM files f
        LEFT JOIN analysis a ON a.file_id = f.id
        WHERE f.app_id = ?
        ORDER BY f.id
        "#
    )
    .bind(app_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| warp::reject())?;

    let page = AnalysisAllPage { app: &app, rows: &rows };
    let html = page.render().map_err(|_| warp::reject())?;
    Ok(warp::reply::with_status(warp::reply::html(html), StatusCode::OK))
}

// ====== API untuk modal: ambil full konten analisa ======
pub async fn api_get_analysis(file_id: i32, kind: String, pool: MySqlPool)
    -> Result<impl Reply, Rejection>
{
    let row = match sqlx::query!(
        "SELECT analisa_fungsi, analisa_relasi_file, analisa_relasi_db FROM analysis WHERE file_id=?",
        file_id
    ).fetch_optional(&pool).await {
        Ok(r) => r,
        Err(e) => {
            let body = json!({"error":"db_error", "message": e.to_string()});
            return Ok(warp::reply::with_status(
                warp::reply::json(&body), StatusCode::INTERNAL_SERVER_ERROR
            ));
        }
    };

    let (title, content) = if let Some(r) = row {
        match kind.as_str() {
            "fungsi"      => ("Analisa Fungsi",     r.analisa_fungsi.unwrap_or("Belum ada hasil.".into())),
            "relasi_file" => ("Relasi File",        r.analisa_relasi_file.unwrap_or("Belum ada hasil.".into())),
            "relasi_db"   => ("Relasi DB",          r.analisa_relasi_db.unwrap_or("Belum ada hasil.".into())),
            _             => ("Tidak dikenal",      "Jenis analisa tidak dikenal".into()),
        }
    } else {
        ("Belum dianalisis", "Belum ada hasil. Jalankan analisa terlebih dahulu.".into())
    };

    let body = json!({ "title": title, "content": content });
    Ok(warp::reply::with_status(warp::reply::json(&body), StatusCode::OK))
}

// ====== Generate graph JS via GPT dan simpan ke files.json_graph ======
pub async fn generate_graph(file_id: i32, pool: MySqlPool) -> Result<impl Reply, Rejection> {
    let row = sqlx::query!(
        "SELECT full_path, content_file FROM files WHERE id=?",
        file_id
    ).fetch_one(&pool).await.map_err(|_| warp::reject())?;

    let code = if let Some(c) = row.content_file {
        c
    } else {
        std::fs::read_to_string(&row.full_path).unwrap_or_default()
    };

    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let js = crate::openai::generate_graph_js(&api_key, &code).await.map_err(|_| warp::reject())?;

    sqlx::query!("UPDATE files SET json_graph=? WHERE id=?", js, file_id)
        .execute(&pool)
        .await
        .map_err(|_| warp::reject())?;

    let body = json!({ "ok": true });
    Ok(warp::reply::with_status(warp::reply::json(&body), StatusCode::OK))
}

// ====== Render graph ======
pub async fn view_graph(file_id: i32, pool: MySqlPool) -> Result<impl Reply, Rejection> {
    let row = sqlx::query!(
        r#"
        SELECT f.nama_file, f.json_graph, a.id as app_id, a.nama_aplikasi, a.created_at
        FROM files f
        JOIN applications a ON a.id = f.app_id
        WHERE f.id=?
        "#,
        file_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| warp::reject())?;

    let Some(r) = row else {
        return Ok(warp::reply::with_status(
            warp::reply::html("File tidak ditemukan".to_string()),
            StatusCode::NOT_FOUND,
        ));
    };

    let app = crate::models::AppRow {
        id: r.app_id as i64,
        nama_aplikasi: r.nama_aplikasi,
        created_at: r.created_at.unwrap_or(Utc::now()),
    };

    let js = r.json_graph.unwrap_or_default();
    if js.trim().is_empty() {
        let html = "<div class='container p-3'><a href='javascript:history.back()'>&larr; Kembali</a><h4>Belum ada graph</h4><p>Silakan klik <em>Generate JSON</em> terlebih dahulu.</p></div>";
        return Ok(warp::reply::with_status(
            warp::reply::html(html.to_string()),
            StatusCode::OK,
        ));
    }

    let page = crate::models::GraphPage {
    app: &app,
    file_name: &r.nama_file,
    graph_js: &js,
};

    let html = page.render().map_err(|_| warp::reject())?;
    Ok(warp::reply::with_status(warp::reply::html(html), StatusCode::OK))
}
pub async fn analyze_file(
    file_id: i32,
    kind: String,
    pool: MySqlPool,
    force: bool,
) -> Result<impl Reply, Rejection> {
    let (app_id,): (i64,) = sqlx::query_as("SELECT app_id FROM files WHERE id=?")
        .bind(file_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| warp::reject())?;
    let back_link = format!("/apps/{app_id}");

    if !force {
        if let Some(row) = sqlx::query!(
            "SELECT analisa_fungsi, analisa_relasi_file, analisa_relasi_db FROM analysis WHERE file_id=?",
            file_id
        )
        .fetch_optional(&pool)
        .await
        .map_err(|_| warp::reject())?
        {
            let cached = match kind.as_str() {
                "fungsi" => row.analisa_fungsi,
                "relasi_file" => row.analisa_relasi_file,
                "relasi_db" => row.analisa_relasi_db,
                _ => None,
            };
            if let Some(content) = cached {
                let page = AnalysisPage {
                    title: &format!("Hasil Analisa {}", kind),
                    content: &content,
                    back_href: &back_link,
                    force_href: Some(&format!("/analyze/{}/{}/force", file_id, kind)),
                };
                return Ok(warp::reply::html(page.render().unwrap()));
            }
        }
    }

    // Ambil konten file dari DB; jika kosong, fallback ke filesystem
    let row = sqlx::query!(
        "SELECT full_path, content_file FROM files WHERE id=?",
        file_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| warp::reject())?;

    let code = if let Some(c) = row.content_file {
        c
    } else {
        std::fs::read_to_string(&row.full_path).unwrap_or_default()
    };

    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

    let result = match kind.as_str() {
        "fungsi" => crate::openai::analisa_fungsi(&api_key, &code).await,
        "relasi_file" => crate::openai::analisa_relasi_file(&api_key, &code).await,
        "relasi_db" => crate::openai::analisa_relasi_db(&api_key, &code).await,
        _ => Ok("Jenis analisa tidak dikenal".to_string()),
    }
    .map_err(|_| warp::reject())?;

    // Simpan/update hasil analisa
    let col = match kind.as_str() {
        "fungsi" => "analisa_fungsi",
        "relasi_file" => "analisa_relasi_file",
        "relasi_db" => "analisa_relasi_db",
        _ => "analisa_fungsi",
    };
    let q = format!(
        "INSERT INTO analysis (file_id, {col}) VALUES (?, ?)
         ON DUPLICATE KEY UPDATE {col}=VALUES({col}), created_at=CURRENT_TIMESTAMP"
    );
    sqlx::query(&q)
        .bind(file_id)
        .bind(&result)
        .execute(&pool)
        .await
        .map_err(|_| warp::reject())?;

    let page = AnalysisPage {
        title: &format!("Hasil Analisa {}", kind),
        content: &result,
        back_href: &back_link,
        force_href: Some(&format!("/analyze/{}/{}/force", file_id, kind)),
    };
    Ok(warp::reply::html(page.render().unwrap()))
}