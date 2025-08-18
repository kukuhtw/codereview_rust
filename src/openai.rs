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

// ======== BARU: generator JS untuk vis-network ========

pub async fn generate_graph_js(api_key: &str, kode: &str) -> Result<String> {
    let prompt = format!(
        concat!(
            "Jelaskan lengkap detail dan buat kode JavaScript (tanpa tag HTML, head, dan body) ",
            "untuk menggambar diagram graph menggunakan vis-network.\n",
            "Pada node, sebutkan semua actor yang terlibat.\n",
            "Pada edges, sebutkan semua event dan atribut. Tambahkan arrow bila ada.\n",
            "   var options = {{\n",
            "       width: '100%',\n",
            "       interaction: {{\n",
            "           dragNodes: true,\n",
            "           dragView: true,\n",
            "           zoomView: true\n",
            "       }},\n",
            "       physics: false,\n",
            "       edges: {{\n",
            "           arrows: {{\n",
            "               to: {{ enabled: true, scaleFactor: 1 }},\n",
            "               from: {{ enabled: false }}\n",
            "           }}\n",
            "       }}\n",
            "   }};\n\n",
            "Berikan new line bila info lebih dari 2 kata.\n\n",
            "Gunakan div dengan id `mynetwork` yang sudah ada di halaman.\n",
            "Gunakan tag ```javascript untuk kode JavaScript.\n",
            "Pastikan width=100%.\n\n",
            "Basiskan diagram pada file berikut:\n{}"
        ),
        kode
    );

    let raw = call_chatgpt(api_key, &prompt).await?;
    // UBAH: kirim &raw, lalu fallback ke raw (memindahkan raw) jika None
    let js = extract_code_block(&raw).unwrap_or(raw);
    Ok(js)
    
    
}

fn extract_code_block(s: &str) -> Option<String> {
    let start_tag = "```javascript";
    let alt_start = "```js";
    let backticks = "```";

    if let Some(start) = s.find(start_tag).or_else(|| s.find(alt_start)) {
        let after = &s[start..];
        let after = after.splitn(2, '\n').nth(1)?; // konten setelah baris pertama
        let end_idx = after.find(backticks)?;
        return Some(after[..end_idx].to_string());
    }
    None
}