// trial-binary/src/main.rs
// Run with: cargo run

use ed25519_dalek::{VerifyingKey, Signature, Verifier};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

// 🔑 EMBED YOUR PUBLIC KEY HERE (get it from license server at startup)
const PUBLIC_KEY_BYTES: [u8; 32] = hex_literal::hex!("b00d8a651dc7702f0e5f3ebc72b3f87aa5e8b8ad482904b9b8954a778ddc6122");
const GRACE_PERIOD_HOURS: u64 = 24; // Allow 24 hours offline
const LAST_CHECK_FILE: &str = ".last_license_check";

#[derive(Serialize, Deserialize)]
struct TrialToken {
    user_id: String,
    issued_at: u64,
    expires_at: u64,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn verify_trial_token(token_json: &str, signature_hex: &str) -> Result<TrialToken, String> {
    // 1. Decode public key
    let verifying_key = VerifyingKey::from_bytes(&PUBLIC_KEY_BYTES)
        .map_err(|e| format!("❌ Invalid public key: {}", e))?;
    
    // 2. Decode signature (trim whitespace first!)
    let sig_bytes = hex::decode(signature_hex.trim())
        .map_err(|e| format!("❌ Invalid signature hex: {}", e))?;
    let sig_array: [u8; 64] = sig_bytes.try_into()
        .map_err(|_| "❌ Signature must be 64 bytes".to_string())?;
    let signature = Signature::from_bytes(&sig_array);
    
    // 3. Verify cryptographic signature
    verifying_key.verify(token_json.as_bytes(), &signature)
        .map_err(|_| "❌ INVALID TOKEN: Signature verification failed! Token was not issued by authorized license server.".to_string())?;
    
    // 4. Parse token
    let token: TrialToken = serde_json::from_str(token_json)
        .map_err(|e| format!("❌ Invalid token format: {}", e))?;
    
    // 5. Check expiry
    let now = current_timestamp();
    if now > token.expires_at {
        let days_ago = (now - token.expires_at) / (24 * 60 * 60);
        return Err(format!(
            "❌ TRIAL EXPIRED: Your trial expired {} days ago. Please contact support to upgrade.",
            days_ago
        ));
    }
    
    // Calculate days remaining
    let seconds_remaining = token.expires_at - now;
    let days_remaining = seconds_remaining / (24 * 60 * 60);
    
    println!("✅ LICENSE VALID");
    println!("   User: {}", token.user_id);
    println!("   Days remaining: {}", days_remaining);
    println!();
    
    Ok(token)
}

fn check_grace_period() -> Result<bool, String> {
    match fs::read_to_string(LAST_CHECK_FILE) {
        Ok(content) => {
            if let Ok(last_check) = content.trim().parse::<u64>() {
                let now = current_timestamp();
                let hours_since_check = (now - last_check) / 3600;
                
                if hours_since_check <= GRACE_PERIOD_HOURS {
                    let hours_remaining = GRACE_PERIOD_HOURS - hours_since_check;
                    println!("   Using offline grace period ({} hours remaining)", hours_remaining);
                    Ok(true)
                } else {
                    Err(format!(
                        "❌ LICENSE CHECK REQUIRED: Last online check was {} hours ago.\n   Please connect to the internet to verify your license.",
                        hours_since_check
                    ))
                }
            } else {
                Err("❌ LICENSE CHECK REQUIRED: Could not read last check timestamp.\n   Please connect to the internet to verify your license.".to_string())
            }
        },
        Err(_) => {
            Err("❌ LICENSE CHECK REQUIRED: No previous online check found.\n   Please connect to the internet to verify your license.".to_string())
        }
    }
}

async fn check_revocation(user_id: &str) -> Result<bool, String> {
    // Check with license server for revocation
    let url = format!("http://127.0.0.1:8081/api/trial/check?user_id={}", user_id);
    
    match reqwest::get(&url).await {
        Ok(response) => {
            // Online: Check revocation and update last check time
            match response.json::<serde_json::Value>().await {
                Ok(data) => {
                    // Save successful check timestamp
                    let _ = fs::write(LAST_CHECK_FILE, current_timestamp().to_string());
                    
                    if data["revoked"].as_bool().unwrap_or(false) {
                        Err("❌ LICENSE REVOKED: Your trial has been revoked by the license server.".to_string())
                    } else {
                        println!("✅ License verified online");
                        Ok(true)
                    }
                },
                Err(e) => {
                    println!("⚠️  Warning: Could not parse server response: {}", e);
                    check_grace_period()
                }
            }
        },
        Err(e) => {
            // Offline: Check if within grace period
            println!("⚠️  Warning: Could not reach license server: {}", e);
            check_grace_period()
        }
    }
}

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║     🚀 TRIAL BINARY v1.0.0             ║");
    println!("╚════════════════════════════════════════╝\n");
    
    // Load token files (trim whitespace!)
    let token_json = match fs::read_to_string("trial.token") {
        Ok(content) => content.trim().to_string(),
        Err(_) => {
            eprintln!("❌ ERROR: trial.token file not found!");
            eprintln!("   Please obtain a trial license from the license server.\n");
            eprintln!("   Run this command:");
            eprintln!("   cargo run --bin get-license -- demo-user\n");
            std::process::exit(1);
        }
    };
    
    let signature_hex = match fs::read_to_string("trial.signature") {
        Ok(content) => content.trim().to_string(),
        Err(_) => {
            eprintln!("❌ ERROR: trial.signature file not found!");
            eprintln!("   Please obtain a trial license from the license server.\n");
            eprintln!("   Run this command:");
            eprintln!("   cargo run --bin get-license -- demo-user\n");
            std::process::exit(1);
        }
    };
    
    // Verify token
    let token = match verify_trial_token(&token_json, &signature_hex) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}\n", e);
            std::process::exit(1);
        }
    };
    
    // Check revocation (online with grace period)
    if let Err(e) = check_revocation(&token.user_id).await {
        eprintln!("{}\n", e);
        std::process::exit(1);
    }
    
    // ✅ License is valid! Run the actual program
    println!("╔════════════════════════════════════════╗");
    println!("║     ✨ LICENSED APPLICATION ✨         ║");
    println!("╚════════════════════════════════════════╝\n");
    
    println!("Hello, {}! 👋", token.user_id);
    println!("Your trial binary is running successfully!");
    println!("\nThis is where your actual application logic would run.");
    println!("Since the license is valid, all features are unlocked.\n");
    
    // Your actual application logic here...
    println!("🎉 Application completed successfully!\n");
}
