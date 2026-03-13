# L-NHI Gateway MVP — API Test Commands

Ensure the server is running: `cd gateway-api && cargo run`

---

## 1. Health check

```bash
curl http://127.0.0.1:3000/
```

Expected: `L-NHI Gateway is running`

---

## 2. Create wallet (Alice)

```bash
curl -X POST http://127.0.0.1:3000/wallets \
  -H "Content-Type: application/json" \
  -d '{"owner_name":"Alice","balance":1000}'
```

Expected: `{"id":"<uuid>","owner_name":"Alice","balance":1000}` — save the `id` as `ALICE_ID`.

---

## 3. Create wallet (Bob)

```bash
curl -X POST http://127.0.0.1:3000/wallets \
  -H "Content-Type: application/json" \
  -d '{"owner_name":"Bob","balance":500}'
```

Expected: `{"id":"<uuid>","owner_name":"Bob","balance":500}` — save the `id` as `BOB_ID`.

---

## 4. Get wallet by ID

```bash
curl http://127.0.0.1:3000/wallets/<ALICE_ID>
```

Replace `<ALICE_ID>` with the UUID from step 2. Expected: `{"id":"...","owner_name":"Alice","balance":1000}`

---

## 5. Get non-existent wallet (404)

```bash
curl -w "\nHTTP %{http_code}\n" http://127.0.0.1:3000/wallets/00000000-0000-0000-0000-000000000000
```

Expected: `wallet not found` with HTTP 404.

---

## 6. Transfer (Alice → Bob)

```bash
curl -X POST http://127.0.0.1:3000/transfer \
  -H "Content-Type: application/json" \
  -d '{"from_wallet_id":"<ALICE_ID>","to_wallet_id":"<BOB_ID>","amount":100}'
```

Replace `<ALICE_ID>` and `<BOB_ID>` with UUIDs from steps 2 and 3.

Expected: `{"status":"completed","from_wallet_id":"...","to_wallet_id":"...","amount":100}`

---

## 7. Verify balances after transfer

```bash
curl http://127.0.0.1:3000/wallets/<ALICE_ID>
curl http://127.0.0.1:3000/wallets/<BOB_ID>
```

Expected: Alice balance 900, Bob balance 600.

---

## 8. Rejected transfer (insufficient balance)

```bash
curl -X POST http://127.0.0.1:3000/transfer \
  -H "Content-Type: application/json" \
  -d '{"from_wallet_id":"<BOB_ID>","to_wallet_id":"<ALICE_ID>","amount":9999}'
```

Expected: `insufficient balance` with HTTP 400. An audit log with `status=rejected` is written.

---

## 9. Rejected transfer (invalid amount)

```bash
curl -X POST http://127.0.0.1:3000/transfer \
  -H "Content-Type: application/json" \
  -d '{"from_wallet_id":"<ALICE_ID>","to_wallet_id":"<BOB_ID>","amount":0}'
```

Expected: `amount must be positive` with HTTP 400.
