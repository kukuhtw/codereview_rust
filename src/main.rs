// src/main.rs
/*
cd /rust/codereview
cargo clean
cargo build
cargo run

/rust/codereview/cargo clean

/rust/codereview/cargo build

*/
// src/main.rs
/*
cd /rust/codereview
cargo clean
cargo build
cargo run
*/
// src/main.rs
/*
cd /rust/codereview
cargo clean
cargo build

*/

mod db;
mod services;
mod openai;
mod handlers;
mod models;

use std::convert::Infallible;
use warp::{http::StatusCode, Filter, Rejection, Reply};
use std::collections::HashMap; // ⟵ TAMBAH

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let pool = db::init_pool().await;

    // GET /health
    let health = warp::path!("health")
        .and(warp::get())
        .and(warp::path::end())
        .map(|| warp::reply::with_status("OK", StatusCode::OK));

    // GET /favicon.ico (agar browser tidak bikin 404 di log)
    let favicon = warp::path("favicon.ico")
        .and(warp::get())
        .map(|| warp::reply::with_status("no favicon", StatusCode::NOT_FOUND));

    // GET /
    let index = warp::path::end()
        .and(with_db(pool.clone()))
        .and_then(|pool| handlers::list_apps(pool));

    // GET /upload
    let upload_page = warp::path("upload")
        .and(warp::get())
        .and_then(handlers::upload_page);

    // POST /upload
    let upload_post = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000))
        .and(with_db(pool.clone()))
        .and_then(handlers::upload_zip);

    // GET /apps  (WAJIB pakai path::end() supaya tak bentrok dengan /apps/:id)
    let apps_index = warp::path("apps")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(handlers::list_apps);

  let app_detail = warp::path!("apps" / i32)
    .and(warp::get())
    .and(warp::query::<HashMap<String, String>>())
    .and(with_db(pool.clone()))
    .and_then(|id, qs: HashMap<String,String>, pool| {
        let page = qs.get("page").and_then(|v| v.parse::<usize>().ok());
        let q = qs.get("q").cloned();
        let qobj = handlers::PageQ { page, q };
        handlers::app_detail(id, qobj, pool)
    });



    // GET /analyze/:file_id/:kind
    let analyze = warp::path!("analyze" / i32 / String)
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(|id, kind, pool| handlers::analyze_file(id, kind, pool, false));

    // GET /analyze/:file_id/:kind/force
    let analyze_force = warp::path!("analyze" / i32 / String / "force")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(|id, kind, pool| handlers::analyze_file(id, kind, pool, true));

    // GET /apps/:id/summary
    let summary = warp::path!("apps" / i32 / "summary")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(|id, pool| handlers::app_summary(id, pool, false));

    // GET /apps/:id/summary/force
    let summary_force = warp::path!("apps" / i32 / "summary" / "force")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(|id, pool| handlers::app_summary(id, pool, true));

    // src/main.rs (di dalam main())

// GET /apps/:id/analysis   ← halaman semua analisa
// GET /apps/:id/analysis
    let analysis_all = warp::path!("apps" / i32 / "analysis")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(handlers::app_analysis_all);

  // GET /api/analysis/:file_id/:kind
    

    let api_analysis = warp::path!("api" / "analysis" / i32 / String)
    .and(warp::get())
    .and(with_db(pool.clone()))
    .and_then(handlers::api_get_analysis);

    // ==== BARU: generate graph dan view graph ====
    // POST /files/:id/generate_graph
    let generate_graph = warp::path!("files" / i32 / "generate_graph")
        .and(warp::post())
        .and(with_db(pool.clone()))
        .and_then(handlers::generate_graph);

    // GET /files/:id/graph
    let view_graph = warp::path!("files" / i32 / "graph")
        .and(warp::get())
        .and(with_db(pool.clone()))
        .and_then(handlers::view_graph);

   // GET /api/apps/:id/summary_preview  ← ringkasan 50 kata untuk modal
let api_app_summary_preview = warp::path!("api" / "apps" / i32 / "summary_preview")
    .and(warp::get())
    .and(with_db(pool.clone()))
    .and_then(handlers::api_get_app_summary_preview);

// GET /api/apps/:id/summary  ← konten utuh untuk modal
let api_app_summary_full = warp::path!("api" / "apps" / i32 / "summary")
    .and(warp::get())
    .and(with_db(pool.clone()))
    .and_then(handlers::api_get_app_summary_full);


    // SATU-SATUNYA komposisi routes
    let routes = favicon
        .or(health)
        .or(index)
        .or(upload_page)
        .or(upload_post)
        .or(apps_index)
        .or(app_detail)
        .or(analysis_all)     // ⟵ masukkan route baru di sini
        .or(analyze)
        .or(api_analysis)
        .or(generate_graph)
        .or(view_graph)
        .or(analyze_force)
        .or(summary)
        .or(summary_force)
        .or(api_app_summary_full)   // ⟵ tambah ini
        .or(api_app_summary_preview)   // ⟵ tambahkan ini
        .recover(handle_rejection)
        .with(warp::log("code_review_ssr"));

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    println!("SSR running at http://127.0.0.1:{port}");
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

fn with_db(
    pool: sqlx::MySqlPool,
) -> impl Filter<Extract = (sqlx::MySqlPool,), Error = Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    eprintln!("Rejection: {:?}", err);
    let msg = "Terjadi kesalahan di server (lihat log).";
    Ok(warp::reply::with_status(msg, StatusCode::INTERNAL_SERVER_ERROR))
}

async fn api_recover(err: Rejection) -> Result<impl Reply, Infallible> {
    eprintln!("API Rejection: {:?}", err);
    let body = serde_json::json!({
        "error": "rejected",
        "message": "Terjadi kesalahan di server (api)."
    });
    Ok(warp::reply::with_status(warp::reply::json(&body), StatusCode::INTERNAL_SERVER_ERROR))
}
