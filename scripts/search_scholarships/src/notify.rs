use std::env;
use anyhow::Result;

/// Get non-empty environment variable value
fn get_env_non_empty(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.is_empty())
}

pub fn send_notifications(msg: &str) -> Result<()> {
    let telegram_token = get_env_non_empty("TELEGRAM_BOT_TOKEN");
    let telegram_chat = get_env_non_empty("TELEGRAM_CHAT_ID");
    let slack_webhook = get_env_non_empty("SLACK_WEBHOOK_URL");
    let discord_webhook = get_env_non_empty("DISCORD_WEBHOOK_URL");
    
    let mut sent = false;
    
    if let (Some(token), Some(chat_id)) = (telegram_token, telegram_chat) {
        println!("Sending Telegram notification...");
        send_telegram(&token, &chat_id, msg)?;
        sent = true;
    }
    
    if let Some(webhook) = slack_webhook {
        println!("Sending Slack notification...");
        send_slack(&webhook, msg)?;
        sent = true;
    }
    
    if let Some(webhook) = discord_webhook {
        println!("Sending Discord notification...");
        send_discord(&webhook, msg)?;
        sent = true;
    }
    
    if !sent {
        println!("No notification channels configured. Message:\n{}", msg);
    }
    
    Ok(())
}

fn send_telegram(token: &str, chat_id: &str, text: &str) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    client.post(&url)
        .form(&[("chat_id", chat_id), ("text", text)])
        .send()?;
    Ok(())
}

fn send_slack(webhook_url: &str, text: &str) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    client.post(webhook_url)
        .json(&serde_json::json!({"text": text}))
        .send()?;
    Ok(())
}

fn send_discord(webhook_url: &str, text: &str) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    client.post(webhook_url)
        .json(&serde_json::json!({"content": text}))
        .send()?;
    Ok(())
}
