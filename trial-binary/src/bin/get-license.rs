// trial-binary/src/bin/get-license.rs
// Run with: cargo run --bin get-license -- demo-user3

use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Deserialize)]
struct LicenseResponse {
    token: String,
    signature: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let user_id = args.get(1).map(|s| s.as_str()).unwrap_or("demo-user");
    
    println!("🔄 Requesting license for: {}", user_id);
    
    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8081/api/trial/issue")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "user_id": user_id
        }))
        .send()
        .await?;
    
    if !response.status().is_success() {
        eprintln!("❌ Error: Server returned status {}", response.status());
        eprintln!("   Response: {}", response.text().await?);
        std::process::exit(1);
    }
    
    let license: LicenseResponse = response.json().await?;
    
    // Write files
    fs::write("trial.token", &license.token)?;
    fs::write("trial.signature", &license.signature)?;
    
    println!("✅ License files created successfully!");
    println!("   trial.token");
    println!("   trial.signature");
    println!("\n🚀 Now run: cargo run");
    
    Ok(())
}
