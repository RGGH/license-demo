// license-server/src/main.rs
// Run with: cargo run

use actix_web::{web, App, HttpServer, HttpResponse, Result};
use ed25519_dalek::{SigningKey, Signer, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
struct TrialToken {
    user_id: String,
    issued_at: u64,
    expires_at: u64,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
    signature: String,
    message: String,
}

#[derive(Deserialize)]
struct IssueRequest {
    user_id: String,
}

#[derive(Deserialize)]
struct CheckRequest {
    user_id: String,
}

#[derive(Serialize)]
struct CheckResponse {
    revoked: bool,
    message: String,
}

// Simple in-memory "database" for demo
struct AppState {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
    revoked_users: Mutex<HashMap<String, bool>>,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// POST /api/trial/issue
async fn issue_trial(
    data: web::Json<IssueRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let now = current_timestamp();
    let token = TrialToken {
        user_id: data.user_id.clone(),
        issued_at: now,
        expires_at: now + (14 * 24 * 60 * 60), // 14 days in seconds
    };
    
    let token_json = serde_json::to_string(&token).unwrap();
    let signature = state.signing_key.sign(token_json.as_bytes());
    
    println!("✓ Issued trial for user: {}", data.user_id);
    println!("  Expires: {} seconds from now", 14 * 24 * 60 * 60);
    
    Ok(HttpResponse::Ok().json(TokenResponse {
        token: token_json,
        signature: hex::encode(signature.to_bytes()),
        message: format!("Trial issued for {} (14 days)", data.user_id),
    }))
}

// GET /api/trial/check?user_id=xxx
async fn check_revocation(
    query: web::Query<CheckRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let revoked = state.revoked_users
        .lock()
        .unwrap()
        .get(&query.user_id)
        .copied()
        .unwrap_or(false);
    
    let message = if revoked {
        format!("User {} has been revoked", query.user_id)
    } else {
        format!("User {} is active", query.user_id)
    };
    
    Ok(HttpResponse::Ok().json(CheckResponse {
        revoked,
        message,
    }))
}

// POST /api/trial/revoke
async fn revoke_trial(
    data: web::Json<IssueRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    state.revoked_users
        .lock()
        .unwrap()
        .insert(data.user_id.clone(), true);
    
    println!("✗ Revoked trial for user: {}", data.user_id);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Trial revoked for {}", data.user_id)
    })))
}

// GET /api/public-key
async fn get_public_key(state: web::Data<AppState>) -> Result<HttpResponse> {
    let public_key_bytes = state.verifying_key.to_bytes();
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "public_key": hex::encode(public_key_bytes),
        "format": "ed25519",
        "note": "Embed this in your trial binary"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🔐 License Server Starting...\n");
    
    // Generate keypair (in production, load from secure storage!)
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    
    println!("📝 Your Public Key (embed this in trial binary):");
    println!("   {}\n", hex::encode(verifying_key.to_bytes()));
    
    let app_state = web::Data::new(AppState {
        signing_key,
        verifying_key,
        revoked_users: Mutex::new(HashMap::new()),
    });
    
    println!("🚀 Server running at http://127.0.0.1:8081\n");
    println!("Available endpoints:");
    println!("  POST   /api/trial/issue     - Issue new trial");
    println!("  GET    /api/trial/check     - Check revocation status");
    println!("  POST   /api/trial/revoke    - Revoke a trial");
    println!("  GET    /api/public-key      - Get public key\n");
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/api/trial/issue", web::post().to(issue_trial))
            .route("/api/trial/check", web::get().to(check_revocation))
            .route("/api/trial/revoke", web::post().to(revoke_trial))
            .route("/api/public-key", web::get().to(get_public_key))
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
