# L-NHI Gateway MVP

A minimal Rust backend for an L-NHI Gateway. It provides wallet creation, lookup, and transfer between wallets with audit logging.

## Why This Project Exists

This repository is a proof-of-concept backend for an L-NHI (Local National Health Identifier) Gateway. The goal is to validate core wallet and transfer flows before integrating with external systems. It serves as a local development and testing foundation.

## Current Features

- **Health check** — `GET /` returns a simple status message
- **Wallet creation** — `POST /wallets` creates a wallet with owner name and initial balance
- **Wallet lookup** — `GET /wallets/:id` fetches a wallet by UUID
- **Transfer** — `POST /transfer` moves balance between wallets with validation
- **Audit logging** — All wallet creation and transfer attempts (success or rejection) are recorded in `audit_logs`

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Web framework | Axum |
| Async runtime | Tokio |
| Database | PostgreSQL |
| DB access | SQLx |
| Config | dotenvy |
| Serialization | Serde |
| IDs | UUID |

## Project Structure

```
l-nhi-gateway-mvp/
├── README.md
├── SETUP.md           # Detailed setup and troubleshooting
├── API_TESTS.md       # Curl commands for testing endpoints
└── gateway-api/
    ├── Cargo.toml
    ├── .env            # DATABASE_URL (not committed)
    ├── .gitignore
    ├── migrations/
    │   └── 20240313000001_init.sql
    └── src/
        ├── main.rs     # Entry point, minimal
        ├── db.rs       # AppState, pool setup
        ├── models.rs   # Request/response structs
        ├── routes.rs   # Route registration
        └── handlers/
            ├── mod.rs
            ├── wallet.rs   # create_wallet, get_wallet
            └── transfer.rs # transfer
```

## Local Setup

**Prerequisites:** Rust, PostgreSQL, `sqlx-cli`

1. Start PostgreSQL (if not running).
2. Create the database:
   ```bash
   createdb l_nhi_gateway
   ```
3. Go to the API crate and configure:
   ```bash
   cd gateway-api
   ```
   Ensure `.env` contains:
   ```
   DATABASE_URL=postgres://aiden@localhost/l_nhi_gateway
   ```
4. Install `sqlx-cli` (one-time):
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   ```
5. Run migrations:
   ```bash
   sqlx migrate run
   ```
6. Start the server:
   ```bash
   cargo run
   ```

The server listens on `http://127.0.0.1:3000`. See [SETUP.md](SETUP.md) for detailed steps and common errors.

## API Reference

| Method | Path | Description |
|--------|------|--------------|
| GET | `/` | Health check |
| POST | `/wallets` | Create wallet |
| GET | `/wallets/:id` | Get wallet by UUID |
| POST | `/transfer` | Transfer between wallets |

### Example curl Commands

**Health check:**
```bash
curl http://127.0.0.1:3000/
```

**Create wallet:**
```bash
curl -X POST http://127.0.0.1:3000/wallets \
  -H "Content-Type: application/json" \
  -d '{"owner_name":"Alice","balance":1000}'
```

**Get wallet:**
```bash
curl http://127.0.0.1:3000/wallets/<WALLET_ID>
```

**Transfer:**
```bash
curl -X POST http://127.0.0.1:3000/transfer \
  -H "Content-Type: application/json" \
  -d '{"from_wallet_id":"<FROM_ID>","to_wallet_id":"<TO_ID>","amount":100}'
```

More examples: [API_TESTS.md](API_TESTS.md)

## Example Validated Flow

1. Create Alice: `POST /wallets` with `{"owner_name":"Alice","balance":1000}` → returns `id: alice-uuid`
2. Create Bob: `POST /wallets` with `{"owner_name":"Bob","balance":500}` → returns `id: bob-uuid`
3. Transfer 100 from Alice to Bob: `POST /transfer` with `from_wallet_id`, `to_wallet_id`, `amount: 100`
4. Verify: `GET /wallets/alice-uuid` → balance 900; `GET /wallets/bob-uuid` → balance 600

Transfer validation includes: amount > 0, source and destination exist, sufficient balance, and no self-transfer. Failed attempts are logged with `status=rejected` and a reason.

## Database Tables

**wallets**

| Column | Type | Notes |
|--------|------|-------|
| id | UUID | Primary key |
| owner_name | TEXT | Not null |
| balance | BIGINT | Not null, >= 0 |

**audit_logs**

| Column | Type | Notes |
|--------|------|-------|
| id | UUID | Primary key |
| wallet_id | UUID | Nullable |
| action | TEXT | e.g. `create`, `transfer` |
| amount | BIGINT | Nullable |
| status | TEXT | `completed` or `rejected` |
| reason | TEXT | Nullable, for rejections |
| created_at | TIMESTAMPTZ | Default NOW() |

## Current Limitations

- No authentication or authorization
- No rate limiting
- No pagination for lists (no list endpoint yet)
- Single-node only; no distributed deployment
- Local development focus; not hardened for production

## Roadmap

- [ ] Add `GET /wallets` (list wallets with optional pagination)
- [ ] Add authentication (API key or JWT)
- [ ] Add request validation middleware
- [ ] Add integration tests
- [ ] Add Docker support for local dev

## Vision

This MVP is the first step toward a production L-NHI Gateway. The next phase will add policy checks, external integrations, and deployment tooling while keeping the core wallet and transfer logic stable.
