
# codereview\_rust

**codereview\_rust** is an AI-powered code reviewer built with **Rust** that leverages **GPT-5** to read, map, and explain entire codebases—without manually tracing thousands of lines.

**Workflow:** Upload ZIP → Extract → Click **Review** → Get a structured report (files, relationships, database usage, and call flows).

> Perfect for onboarding, legacy audits, large refactors, and knowledge transfer.

---

## 🎬 Demo

Watch the quick demo on YouTube:
**[https://www.youtube.com/watch?v=\_y08n-yFXY8](https://www.youtube.com/watch?v=_y08n-yFXY8)**

---

## ✨ Features

* **ZIP Upload**

  * Upload a single `.zip` with your entire source (Rust, PHP, JS/TS, Python, Go, Java, SQL, etc.).
  * Auto-extraction + indexing (path, extension, size, hash, safe content snippet).

* **Codebase Explorer**

  * Browse folder/file structure like a file explorer.
  * Read-only file previews.

* **AI Review (GPT-5)**

  * One-click **Review**:

    * Explains each file’s role.
    * Extracts **classes, functions, methods**.
    * Builds a **dependency graph** across files.
    * Detects **databases, tables, SQL queries**.

* **Structured Report**

  * Per-file explanations.
  * Cross-file relationships and call flows.
  * Database schema summary (tables, relationships, frequent queries).
  * Natural language output for the whole team.

* **Rust-first performance & safety**

  * Async server, efficient I/O, memory safety by design.
  * Large files handled via chunking & streaming.

---

## 🧩 How It Works

1. **Indexing:** Extract ZIP → walk files → store metadata & safe preview snippets.
2. **File Summaries:** GPT-5 produces per-file roles and public APIs.
3. **Relation Mining:** Detect imports, cross-file calls, references → dependency graph.
4. **DB Insight:** Identify SQL/ORM usage → list tables, columns, relations, key queries.
5. **Global Report:** Merge summaries into a cohesive, human-readable report (+ optional refactor hints).

---

## 🛠 Tech Stack

* **Rust**: Tokio (async), Warp (HTTP)
* **Askama**: Server-rendered HTML templates
* **SQLx (MySQL)**: Persistence for projects, files, and analysis artifacts
* **dotenv**: Configuration
* **OpenAI (GPT-5)**: Semantic analysis & summarization

  > You can swap the LLM provider if needed.

---

## ⚙️ Quickstart

### Prerequisites

* Rust toolchain (stable)
* MySQL/MariaDB
* OpenAI API key (or compatible LLM provider)

### 1) Clone

```bash
git clone https://github.com/kukuhtw/codereview_rust.git
cd codereview_rust
```

### 2) Configure `.env`

```env
# Database
DATABASE_URL=mysql://root:password@127.0.0.1:3306/codereview_rust?ssl-mode=DISABLED

# LLM
OPENAI_API_KEY=sk-...

# Server
RUST_LOG=info
PORT=8080

# Upload limits (optional)
MAX_UPLOAD_MB=200
```

### 3) Initialize Database (example schema)

> Adjust to your migrations if you have them (also provided under `docker/sql/001_init.sql`).

```sql
CREATE TABLE apps (
  id           CHAR(36) PRIMARY KEY,
  name         VARCHAR(255) NOT NULL,
  zip_path     VARCHAR(512) NOT NULL,
  created_at   TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE files (
  id           BIGINT AUTO_INCREMENT PRIMARY KEY,
  app_id       CHAR(36) NOT NULL,
  path         VARCHAR(1024) NOT NULL,
  ext          VARCHAR(16),
  size_bytes   BIGINT,
  sha256       CHAR(64),
  preview_text TEXT,
  INDEX (app_id),
  FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE
);

CREATE TABLE analyses (
  id           BIGINT AUTO_INCREMENT PRIMARY KEY,
  app_id       CHAR(36) NOT NULL,
  status       ENUM('queued','running','done','error') DEFAULT 'queued',
  report_json  LONGTEXT,
  created_at   TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (app_id) REFERENCES apps(id) ON DELETE CASCADE
);
```

### 4) Build & Run

```bash
cargo build --release
./target/release/codereview_rust
# or for dev:
cargo run
```

Open: `http://localhost:8080`

---

## 🐳 Run with Docker

This repo includes a `Dockerfile` and `docker-compose.yml`.

```bash
docker compose up -d --build
# App: http://localhost:8080
```

Environment variables:

* `MYSQL_ROOT_PASSWORD` (default: `password`)
* `MYSQL_DATABASE` (default: `codereview_rust`)
* `OPENAI_API_KEY` (required for GPT-5 review)
* `RUST_LOG`, `PORT` (optional)

> The database schema is auto-loaded from `docker/sql/001_init.sql` on first run.

---

## 🔌 API Overview

> Route names may differ if you customize routing; these are typical defaults.

* `GET /health` — server health
* `GET /apps` — list projects
* `POST /apps/upload` — upload `.zip` (multipart form-data, field: `zip_file`)
* `POST /apps/{app_id}/review` — start GPT-5 analysis
* `GET /apps/{app_id}/analysis` — fetch latest report
* `GET /apps/{app_id}/files` — list files in project
* `GET /files/{file_id}` — get file preview/details

**Upload example (curl):**

```bash
curl -X POST http://localhost:8080/apps/upload \
  -F "zip_file=@/path/to/your_project.zip"
```

**Start review:**

```bash
curl -X POST http://localhost:8080/apps/APP_ID/review
```

**Fetch report:**

```bash
curl http://localhost:8080/apps/APP_ID/analysis
```

---

## 🧰 Project Structure (example)

```
src/
  handlers/     # Warp route handlers
  services/     # indexing + analysis pipeline
  openai/       # GPT-5 client + prompts
  db/           # SQLx pool + repositories
  models/       # entities & DTOs
  main.rs
templates/      # Askama HTML templates
public/         # static assets
docker/
  sql/
    001_init.sql
.github/
  workflows/
    ci.yml
```

---

## ✅ Continuous Integration

This repository ships with **GitHub Actions** that:

* spin up MySQL,
* initialize the schema from `docker/sql/001_init.sql`,
* run `cargo fmt`, `clippy`, `build`, and `test`.

Badge is displayed at the top of this README.
If your tests call the LLM, add `OPENAI_API_KEY` in **Settings → Secrets and variables → Actions**.

---

## 🗺 Roadmap

* [ ] Git repository import (HTTPS/SSH)
* [ ] Visual dependency graph (Graphviz/D3)
* [ ] Deeper language parsers (Java/Kotlin/C#/Go/TS/SQL)
* [ ] Prompt caching & cost controls
* [ ] Offline mode (local LLM)
* [ ] Export to PDF/Markdown

Contributions & ideas are welcome!

---

## 🔒 Security & Privacy

* **No code execution**: static/semantic analysis only.
* Avoid uploading highly sensitive code to public LLMs (consider self-hosted LLMs or masking secrets).
* Upload limits and file filters are configurable.

---

## 📸 Screenshots

> Add UI screenshots (e.g., `docs/explorer.png`, `docs/report.png`) to showcase the explorer and sample reports.

---

## 🤝 Contributing

1. Fork this repo
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Commit: `git commit -m "feat: ..."`
4. Push: `git push origin feat/your-feature`
5. Open a Pull Request

---

## 📄 License

MIT (or update to your company’s preferred license).
See `LICENSE`.

---

## 👤 Author

**Kukuh Tripamungkas Wicaksono (Kukuh TW)**
📧 **[kukuhtw@gmail.com](mailto:kukuhtw@gmail.com)** · 📱 **[https://wa.me/628129893706](https://wa.me/628129893706)**
🔗 **LinkedIn:** [https://id.linkedin.com/in/kukuhtw](https://id.linkedin.com/in/kukuhtw)

---
