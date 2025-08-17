// src/openai.rs
/*
=============================================================================
Project : AI CodeReview Rust
Author : Kukuh Tripamungkas Wicaksono (Kukuh TW)
Email : kukuhtw@gmail.com
WhatsApp : https://wa.me/628129893706
LinkedIn : https://id.linkedin.com/in/kukuhtw
=============================================================================/

*/
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;

async fn call_chatgpt(api_key: &str, prompt: &str) -> Result<String> {
    if api_key.is_empty() {
        // biar errornya jelas kalau lupa set OPENAI_API_KEY
        anyhow::bail!("OPENAI_API_KEY kosong");
    }

    let client = Client::new();
    let resp: serde_json::Value = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": "gpt-5",
            "messages": [
                {"role": "system", "content": "Anda adalah code reviewer profesional."},
                {"role": "user", "content": prompt}
            ]
        }))
        .send()
        .await
        .context("gagal mengirim request ke OpenAI")?
        .json()
        .await
        .context("gagal parsing JSON balasan OpenAI")?;

    // aman-kan akses ke choices[0].message.content
    let content = resp
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.get(0))
        .and_then(|c0| c0.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|s| s.as_str())
        .unwrap_or("");

    Ok(content.to_string())
}

pub async fn analisa_fungsi(api_key: &str, kode: &str) -> Result<String> {
    call_chatgpt(api_key, &format!("Jelaskan fungsi utama file ini:\n{}", kode)).await
}

pub async fn analisa_relasi_file(api_key: &str, kode: &str) -> Result<String> {
    call_chatgpt(
        api_key,
        &format!("File ini menggunakan/memanggil file lain apa saja? Jelaskan:\n{}", kode),
    )
    .await
}

pub async fn analisa_relasi_db(api_key: &str, kode: &str) -> Result<String> {
    call_chatgpt(
        api_key,
        &format!("Database apa yang terlibat di file ini? Cari query SQL / koneksi DB:\n{}", kode),
    )
    .await
}

pub async fn analisa_file(api_key: &str, kode: &str) -> Result<String> {
    call_chatgpt(api_key, &format!("Analisa file berikut:\n{}", kode)).await
}

pub async fn summary_app(api: &str, payload: &str) -> Result<String> {
    call_chatgpt(
        api,
        &format!(
            "Analisa payload aplikasi berikut dan buat ringkasan:\n\
             1) hitung jumlah file; 2) sebutkan tabel & field database; \
             3) identifikasi file paling core/vital; 4) ringkas arsitektur.\n\n{}",
            payload
        ),
    )
    .await
}
