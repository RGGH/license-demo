# Trial License System - Local Demo Setup

## 📁 Project Structure

```
license-demo/
├── license-server/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── trial-binary/
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## Step-by-Step Demo

### Step 1: Create License Server

```bash
# Create project
cargo new license-server
cd license-server

# Copy the license-server code from artifact
# Then run it
cargo run
```

The server will:
- Generate a new keypair
- Print the **public key** (you'll need this!)
- Start on `http://127.0.0.1:8081`

**Copy the public key from the output!** It looks like:
```
📝 Your Public Key (embed this in trial binary):
   a1b2c3d4e5f6...
```

---

### Step 2: Create Trial Binary

```bash
# In parent directory
cd ..
cargo new trial-binary
cd trial-binary

# Copy the trial-binary code from artifact
```

**IMPORTANT:** Edit `trial-binary/src/main.rs` and replace this line:
```rust
const PUBLIC_KEY_BYTES: [u8; PUBLIC_KEY_LENGTH] = [0u8; PUBLIC_KEY_LENGTH];
```

With your actual public key. You can get it via:
```bash
curl http://127.0.0.1:8081/api/public-key
```

Convert the hex string to bytes array using this helper:
```rust
// Helper to convert hex string to byte array
const PUBLIC_KEY_BYTES: [u8; 32] = hex_literal::hex!("YOUR_HEX_KEY_HERE");

// Add to Cargo.toml dependencies:
// hex-literal = "0.4"
```

OR use this simpler approach in your code:
```rust
// At the top of main()
let public_key_hex = "YOUR_HEX_FROM_SERVER";
let PUBLIC_KEY_BYTES = hex::decode(public_key_hex).unwrap();
```

---

### Step 3: Issue a Trial License

```bash
# Request a trial token for user "demo-user"
curl -X POST http://127.0.0.1:8081/api/trial/issue \
     -H 'Content-Type: application/json' \
     -d '{"user_id":"demo-user"}' \
     | jq . > license_response.json

# Extract token and signature to separate files
cat license_response.json | jq -r '.token' > trial.token
cat license_response.json | jq -r '.signature' > trial.signature
```

You should now have:
- `trial.token` - Contains the trial information (JSON)
- `trial.signature` - The cryptographic signature

**Move these files to the trial-binary directory:**
```bash
mv trial.token trial.signature ../trial-binary/
```

---

### Step 4: Run the Trial Binary

```bash
cd ../trial-binary
cargo run
```

You should see:
```
╔════════════════════════════════════════╗
║     🚀 TRIAL BINARY v1.0.0             ║
╚════════════════════════════════════════╝

✅ LICENSE VALID
   User: demo-user
   Days remaining: 14

╔════════════════════════════════════════╗
║     ✨ LICENSED APPLICATION ✨         ║
╚════════════════════════════════════════╝

Hello, demo-user! 👋
Your trial binary is running successfully!
```

---

## 🧪 Testing Different Scenarios

### Test 1: Valid License ✅
```bash
cargo run  # Should work perfectly
```

### Test 2: Revoke a License ❌
```bash
# In another terminal
curl -X POST http://127.0.0.1:8081/api/trial/revoke \
     -H 'Content-Type: application/json' \
     -d '{"user_id":"demo-user"}'

# Now try running the binary again
cargo run  # Should fail: LICENSE REVOKED
```

### Test 3: Tamper with Token
```bash
# Edit trial.token and change user_id
# Then try running
cargo run  # Should fail: Signature verification failed!
```

### Test 4: Check License Status 🔍
```bash
curl 'http://127.0.0.1:8081/api/trial/check?user_id=demo-user'
```

### Test 5: Expired License
To test expiry, modify the license server code to issue shorter trials:
```rust
expires_at: now + (60), // 60 seconds instead of 14 days
```

Then reissue a token and wait 60 seconds before running the binary.

---

## 🔑 API Endpoints

### License Server (Port 8081)

| Method | Endpoint | Purpose |
|--------|----------|---------|
| POST | `/api/trial/issue` | Issue new trial token |
| GET | `/api/trial/check?user_id=X` | Check if revoked |
| POST | `/api/trial/revoke` | Revoke a trial |
| GET | `/api/public-key` | Get public key |

---

## 🛠️ Production Considerations

For a real deployment, you'd want to:

1. **Secure Private Key Storage**
   - Use environment variables
   - Use AWS KMS, Azure Key Vault, or HashiCorp Vault
   - Never commit the private key to git

2. **Persistent Database**
   - Replace in-memory HashMap with PostgreSQL/MySQL
   - Store issued tokens and revocation status

3. **HTTPS**
   - Use TLS for license server endpoints
   - Prevent man-in-the-middle attacks

4. **Rate Limiting**
   - Prevent abuse of trial issuance endpoint
   - Limit revocation checks

5. **Token Storage in Binary**
   - Store in user's config directory
   - Add file permissions restrictions
   - Consider encrypted storage

6. **Graceful Degradation**
   - Allow X hours of offline usage before requiring server check
   - Cache revocation status temporarily

7. **Build the Binary**
   ```bash
   cargo build --release
   # Binary will be in target/release/trial-binary
   ```

---

## 🎯 What This Demonstrates

✅ **Cryptographic Security** - Can't forge tokens without private key
✅ **Time-Limited Trials** - Automatic expiry after 14 days
✅ **Remote Revocation** - Instantly revoke via server
✅ **No Embedded Secrets** - Only public key in binary
✅ **Tamper Detection** - Any modification breaks signature
✅ **Offline Verification** - Works without network (with cached status)

---

## Quick Reference

**Issue new trial:**
```bash
curl -X POST http://127.0.0.1:8081/api/trial/issue \
     -H 'Content-Type: application/json' \
     -d '{"user_id":"USERNAME"}' | jq .
```

**Revoke trial:**
```bash
curl -X POST http://127.0.0.1:8081/api/trial/revoke \
     -H 'Content-Type: application/json' \
     -d '{"user_id":"USERNAME"}'
```

**Check status:**
```bash
curl 'http://127.0.0.1:8081/api/trial/check?user_id=USERNAME'
```

---

## Troubleshooting

**"trial.token file not found"**
- Make sure you've issued a token and moved the files to the binary directory

**"Invalid public key"**
- You forgot to update PUBLIC_KEY_BYTES with the actual public key from the server

**"Could not reach license server"**
- The license server isn't running on port 8081
- This is a warning; offline verification will still work

**"Signature verification failed"**
- Token was tampered with, or
- Public key doesn't match the private key that signed the token

## On trial binary:

```bash
❯ curl -X POST http://127.0.0.1:8081/api/trial/issue \
     -H 'Content-Type: application/json' \
     -d '{"user_id":"demo-user2"}' \
     | jq . > license_response.json
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100   308  100   284  100    24  48168   4070 --:--:-- --:--:-- --:--:-- 61600
❯ cat license_response.json | jq -r '.token' > trial.token
cat license_response.json | jq -r '.signature' > trial.signature
❯ cat license_response.json
{
  "token": "{\"user_id\":\"demo-user2\",\"issued_at\":1766416751,\"expires_at\":1767626351}",
  "signature": "8956dbdbe112e319355ae9fb62235263b6bb61f254059d8aeffdf853fba8f715230e2b41a39afa5f5e8f9a689f005821deaf32621448ad395641bb6140b4f408",
  "message": "Trial issued for demo-user2 (14 days)"
}
❯ cargo r
   Compiling trial-binary v0.1.0 (/home/oem/rust/license-demo/trial-binary)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.74s
     Running `target/debug/trial-binary`
╔════════════════════════════════════════╗
║     🚀 TRIAL BINARY v1.0.0             ║
╚════════════════════════════════════════╝

✅ LICENSE VALID
   User: demo-user2
   Days remaining: 13

╔════════════════════════════════════════╗
║     ✨ LICENSED APPLICATION ✨         ║
╚════════════════════════════════════════╝

Hello, demo-user2! 👋
Your trial binary is running successfully!

This is where your actual application logic would run.
Since the license is valid, all features are unlocked.

🎉 Application completed successfully!

```
