use std::env;
use anyhow::Result;

pub fn send_notifications(msg: &str) -> Result<()> {
    let telegram_token = env::var("TELEGRAM_BOT_TOKEN").ok();
    let telegram_chat = env::var("TELEGRAM_CHAT_ID").ok();
    let slack_webhook = env::var("SLACK_WEBHOOK_URL").ok();
    let discord_webhook = env::var("DISCORD_WEBHOOK_URL").ok();
    
    let mut sent = false;
    
    if let (Some(token), Some(chat_id)) = (telegram_token, telegram_chat) {
        send_telegram(&token, &chat_id, msg)?;
        sent = true;
    }
    
    if let Some(webhook) = slack_webhook {
        send_slack(&webhook, msg)?;
        sent = true;
    }
    
    if let Some(webhook) = discord_webhook {
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
