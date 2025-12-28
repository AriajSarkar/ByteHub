use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN required");
    let app_id = env::var("DISCORD_APPLICATION_ID").expect("DISCORD_APPLICATION_ID required");

    // All commands are guild-only (dm_permission: false, contexts: [0] = Guild context only)
    // This prevents commands from appearing in DMs at all
    let commands = serde_json::json!([
        {
            "name": "submit-project",
            "description": "Submit a GitHub repository for mod approval",
            "dm_permission": false,
            "contexts": [0],
            "options": [
                {
                    "name": "repo",
                    "description": "GitHub repo (e.g. owner/repo-name)",
                    "type": 3,
                    "required": true
                }
            ]
        },
        {
            "name": "approve",
            "description": "Approve a submitted project (mod only)",
            "dm_permission": false,
            "contexts": [0],
            "options": [
                {
                    "name": "repo",
                    "description": "GitHub repo to approve",
                    "type": 3,
                    "required": true
                }
            ]
        },
        {
            "name": "deny",
            "description": "Deny/remove a project (mod only)",
            "dm_permission": false,
            "contexts": [0],
            "options": [
                {
                    "name": "repo",
                    "description": "GitHub repo to deny",
                    "type": 3,
                    "required": true
                }
            ]
        },
        {
            "name": "whitelist-user",
            "description": "Add a GitHub user to the whitelist (mod only)",
            "dm_permission": false,
            "contexts": [0],
            "options": [
                {
                    "name": "username",
                    "description": "GitHub username",
                    "type": 3,
                    "required": true
                }
            ]
        },
        {
            "name": "list",
            "description": "List all registered projects (mod only)",
            "dm_permission": false,
            "contexts": [0]
        },
        {
            "name": "setup-server",
            "description": "Set up ByteHub channels in this server (mod only)",
            "dm_permission": false,
            "contexts": [0]
        }
    ]);

    let url = format!(
        "https://discord.com/api/v10/applications/{}/commands",
        app_id
    );

    let client = reqwest::Client::new();
    let res = client
        .put(&url)
        .header("Authorization", format!("Bot {}", token))
        .header("Content-Type", "application/json")
        .json(&commands)
        .send()
        .await?;

    if res.status().is_success() {
        println!("✅ Commands registered successfully!");
        let body: serde_json::Value = res.json().await?;
        println!(
            "Registered {} commands",
            body.as_array().map(|a| a.len()).unwrap_or(0)
        );
    } else {
        let status = res.status();
        let body = res.text().await?;
        eprintln!("❌ Failed to register commands: {} - {}", status, body);
    }

    Ok(())
}
