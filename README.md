
### `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
  pull_request:

jobs:
  build-test:
    runs-on: ubuntu-latest

    services:
      mysql:
        image: mysql:8
        env:
          MYSQL_ROOT_PASSWORD: password
          MYSQL_DATABASE: codereview_rust
        options: >-
          --health-cmd="mysqladmin ping -h 127.0.0.1 -ppassword --silent"
          --health-interval=5s
          --health-timeout=3s
          --health-retries=20
        ports:
          - 3306:3306

    env:
      DATABASE_URL: mysql://root:password@127.0.0.1:3306/codereview_rust?ssl-mode=DISABLED
      RUST_LOG: info
      # If your tests need the LLM, set this in repo secrets:
      # OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust (stable)
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Install MySQL client
        run: |
          sudo apt-get update
          sudo apt-get install -y mysql-client

      - name: Wait for DB
        run: |
          for i in {1..60}; do
            mysqladmin ping -h 127.0.0.1 -ppassword --silent && break
            sleep 2
          done

      - name: Initialize schema
        run: |
          mysql -h 127.0.0.1 -uroot -ppassword codereview_rust < docker/sql/001_init.sql

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Lint (clippy)
        run: cargo clippy --all-targets -- -D warnings

      - name: Build
        run: cargo build --locked --all-targets

      - name: Test
        run: cargo test --all -- --nocapture
```

### README badge (add near the top)


**Notes**

* The workflow expects your schema at `docker/sql/001_init.sql` (already provided in our earlier steps).
* If any tests call the LLM, add `OPENAI_API_KEY` in **Settings → Secrets and variables → Actions**.
