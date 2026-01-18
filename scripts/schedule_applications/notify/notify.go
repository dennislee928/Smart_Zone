package notify

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"os"
)

func SendNotifications(msg string) error {
	var sent bool

	// Telegram
	token := os.Getenv("TELEGRAM_BOT_TOKEN")
	chatID := os.Getenv("TELEGRAM_CHAT_ID")
	if token != "" && chatID != "" {
		if err := sendTelegram(token, chatID, msg); err == nil {
			sent = true
		}
	}

	// Slack
	slackWebhook := os.Getenv("SLACK_WEBHOOK_URL")
	if slackWebhook != "" {
		if err := sendSlack(slackWebhook, msg); err == nil {
			sent = true
		}
	}

	// Discord
	discordWebhook := os.Getenv("DISCORD_WEBHOOK_URL")
	if discordWebhook != "" {
		if err := sendDiscord(discordWebhook, msg); err == nil {
			sent = true
		}
	}

	if !sent {
		fmt.Println("No notification channels configured. Message:\n" + msg)
	}

	return nil
}

func sendTelegram(token, chatID, text string) error {
	url := fmt.Sprintf("https://api.telegram.org/bot%s/sendMessage", token)
	data := map[string]string{
		"chat_id": chatID,
		"text":    text,
	}
	
	jsonData, _ := json.Marshal(data)
	resp, err := http.Post(url, "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	return nil
}

func sendSlack(webhookURL, text string) error {
	data := map[string]string{
		"text": text,
	}
	
	jsonData, _ := json.Marshal(data)
	resp, err := http.Post(webhookURL, "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	return nil
}

func sendDiscord(webhookURL, text string) error {
	data := map[string]string{
		"content": text,
	}
	
	jsonData, _ := json.Marshal(data)
	resp, err := http.Post(webhookURL, "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	return nil
}
