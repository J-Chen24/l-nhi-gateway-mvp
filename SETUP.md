# L-NHI Gateway MVP — Setup & Run Guide

## Section 1: Overview

This project sets up a minimal Rust backend MVP with:
- **Axum** HTTP server on `127.0.0.1:3000`
- **PostgreSQL** database `l_nhi_gateway` with `wallets` and `audit_logs` tables
- **SQLx** for migrations and (future) queries
- **dotenvy** for `.env` config

---

## Section 2: Commands to Run (in order)

Run these in your terminal. If PostgreSQL is not running, start it first (see Section 6).

```bash
# 1. Start PostgreSQL (if not already running)
LC_ALL="en_US.UTF-8" /opt/homebrew/opt/postgresql@18/bin/postgres -D /opt/homebrew/var/postgresql@18 &
# Or: brew services start postgresql@18

# 2. Create database (run from any folder)
createdb l_nhi_gateway

# 3. Go to project
cd ~/l-nhi-gateway-mvp/gateway-api

# 4. Build (fetches deps)
cargo build

# 5. Install sqlx-cli (one-time, if not installed)
cargo install sqlx-cli --no-default-features --features postgres

# 6. Run migrations
sqlx migrate run

# 7. Run the server
cargo run
```

Then open: http://127.0.0.1:3000 — you should see: `L-NHI Gateway is running`

---

## Section 3: File Contents (already created)

All files are in place. Key paths:
- `gateway-api/Cargo.toml`
- `gateway-api/.env`
- `gateway-api/migrations/20240313000001_init.sql`
- `gateway-api/src/main.rs`

---

## Section 4: Project Tree

```
l-nhi-gateway-mvp/
├── .git/
├── SETUP.md
└── gateway-api/
    ├── .env
    ├── .gitignore
    ├── Cargo.toml
    ├── migrations/
    │   └── 20240313000001_init.sql
    └── src/
        └── main.rs
```

---

## Section 5: Run Checklist

- [ ] PostgreSQL is running
- [ ] Database `l_nhi_gateway` exists
- [ ] `cd ~/l-nhi-gateway-mvp/gateway-api`
- [ ] `cargo build` succeeds
- [ ] `sqlx migrate run` succeeds
- [ ] `cargo run` starts server
- [ ] `curl http://127.0.0.1:3000` returns `L-NHI Gateway is running`

---

## Section 6: Common Errors and Fixes

| Error | Fix |
|-------|-----|
| `connection to server ... failed: Operation not permitted` or `Is the server running?` | PostgreSQL is not running. Start it with: `LC_ALL="en_US.UTF-8" /opt/homebrew/opt/postgresql@18/bin/postgres -D /opt/homebrew/var/postgresql@18` (or `brew services start postgresql@18`) |
| `database "l_nhi_gateway" already exists` | Safe to ignore. Database is ready. |
| `sqlx: command not found` | Run: `cargo install sqlx-cli --no-default-features --features postgres` |
| `error: failed to connect to database` | Check `.env` has `DATABASE_URL=postgres://aiden@localhost/l_nhi_gateway` and PostgreSQL is running |
| `port 3000 already in use` | Kill the process: `lsof -i :3000` then `kill -9 <PID>` |
| `POST /wallets` returns 404 but `GET /` works | An old gateway-api binary is running (without /wallets routes). Kill it: `pkill -9 gateway-api` or `kill -9 $(lsof -t -i:3000)`, then `cargo run` again. |
