// src/services.rs
/*
=============================================================================
Project : AI CodeReview Rust
Author : Kukuh Tripamungkas Wicaksono (Kukuh TW)
Email : kukuhtw@gmail.com
WhatsApp : https://wa.me/628129893706
LinkedIn : https://id.linkedin.com/in/kukuhtw
=============================================================================/

*/
use std::fs::File;
use std::io::Read; // <- untuk read_to_end
use zip::ZipArchive;
use sqlx::MySqlPool;

const MAX_FILE_BYTES: usize = 512 * 1024; // 512 KB per file untuk disimpan ke DB

pub async fn extract_and_store(
    pool: &MySqlPool,
    app_name: &str,
    zip_path: &str,
) -> anyhow::Result<i64> {
    let mut file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(&mut file)?;

    let mut tx = pool.begin().await?;

    // MySQL: pakai last_insert_id
    let res = sqlx::query("INSERT INTO applications (nama_aplikasi) VALUES (?)")
        .bind(app_name)
        .execute(&mut *tx)
        .await?;
    let app_id = res.last_insert_id() as i64;

    for i in 0..archive.len() {
        // Bungkus agar ZipFile drop sebelum .await
        let (nama_file, folder, full_path, content_file): (String, Option<String>, String, Option<String>) = {
            let  entry = archive.by_index(i)?;
            if entry.is_dir() {
                continue;
            }
            let Some(path) = entry.enclosed_name().map(|p| p.to_owned()) else {
                continue;
            };

            // metadata path
            let nama_file = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            let folder = path.parent().map(|p| p.to_string_lossy().into_owned());
            let full_path = path.to_string_lossy().into_owned();

            // baca isi file (dibatasi)
            let mut buf = Vec::new();
            // Hindari membaca file raksasa / biner penuh: stop di MAX_FILE_BYTES
            // ZipFile tidak punya read_exact_at; kita baca sampai limit manual
            let mut limited = entry.take(MAX_FILE_BYTES as u64);
            limited.read_to_end(&mut buf)?;

            // Simpan sebagai UTF-8 (lossy supaya aman untuk file teks campur)
            let content_str = if buf.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&buf).to_string())
            };

            (nama_file, folder, full_path, content_str)
        };

        sqlx::query(
            "INSERT INTO files (app_id, nama_file, nama_folder, full_path, content_file)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(app_id)
        .bind(&nama_file)
        .bind(&folder)
        .bind(&full_path)
        .bind(&content_file)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(app_id)
}
