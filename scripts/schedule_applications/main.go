package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"time"

	"schedule_applications/notify"
	"gopkg.in/yaml.v3"
)

type Application struct {
	Name         string   `json:"name"`
	Deadline     string   `json:"deadline"`
	Status       string   `json:"status"`
	CurrentStage string   `json:"current_stage,omitempty"`
	NextAction   string   `json:"next_action,omitempty"`
	RequiredDocs []string `json:"required_docs,omitempty"`
	Progress     int      `json:"progress,omitempty"`
	Notes        string   `json:"notes,omitempty"`
}

type ApplicationsFile struct {
	Applications []Application `json:"applications"`
}

type Lead struct {
	Name      string `json:"name"`
	Amount    string `json:"amount"`
	Deadline  string `json:"deadline"`
	Status    string `json:"status"`
	Source    string `json:"source"`
	URL       string `json:"url,omitempty"`
}

type LeadsFile struct {
	Leads []Lead `json:"leads"`
}

func main() {
	root := os.Getenv("ROOT")
	if root == "" {
		root = "."
	}

	// Load applications
	appsFile, err := loadApplications(root)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading applications: %v\n", err)
		os.Exit(1)
	}

	// Load leads
	leadsFile, err := loadLeads(root)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading leads: %v\n", err)
		os.Exit(1)
	}

	// Generate schedule suggestions
	suggestions := generateSchedule(appsFile.Applications, leadsFile.Leads)

	// Format message
	msg := formatScheduleMessage(suggestions)

	// Send notifications
	if err := notify.SendNotifications(msg); err != nil {
		fmt.Fprintf(os.Stderr, "Error sending notifications: %v\n", err)
	}

	fmt.Println(msg)
}

func loadApplications(root string) (*ApplicationsFile, error) {
	jsonPath := filepath.Join(root, "tracking", "applications.json")
	yamlPath := filepath.Join(root, "tracking", "applications.yml")

	// Try JSON first
	if data, err := ioutil.ReadFile(jsonPath); err == nil {
		var apps ApplicationsFile
		if err := json.Unmarshal(data, &apps); err == nil {
			return &apps, nil
		}
	}

	// Fallback to YAML
	if data, err := ioutil.ReadFile(yamlPath); err == nil {
		var apps ApplicationsFile
		if err := yaml.Unmarshal(data, &apps); err == nil {
			return &apps, nil
		}
	}

	return &ApplicationsFile{Applications: []Application{}}, nil
}

func loadLeads(root string) (*LeadsFile, error) {
	jsonPath := filepath.Join(root, "tracking", "leads.json")
	
	data, err := ioutil.ReadFile(jsonPath)
	if err != nil {
		return &LeadsFile{Leads: []Lead{}}, nil
	}

	var leads LeadsFile
	if err := json.Unmarshal(data, &leads); err != nil {
		return &LeadsFile{Leads: []Lead{}}, nil
	}

	return &leads, nil
}

func generateSchedule(apps []Application, leads []Lead) []string {
	var suggestions []string
	
	now := time.Now()
	oneWeek := now.Add(7 * 24 * time.Hour)
	
	// Sort applications by deadline
	upcomingApps := []Application{}
	for _, app := range apps {
		if app.Deadline != "" {
			deadline, err := time.Parse("2006-01-02", app.Deadline)
			if err == nil && deadline.After(now) && deadline.Before(oneWeek) {
				upcomingApps = append(upcomingApps, app)
			}
		}
	}
	
	// Add upcoming applications
	for _, app := range upcomingApps {
		daysLeft := int(time.Until(parseDate(app.Deadline)).Hours() / 24)
		suggestions = append(suggestions, fmt.Sprintf("- %s (D-%d days)", app.Name, daysLeft))
	}
	
	// Add qualified leads that haven't been applied
	qualifiedLeads := []Lead{}
	for _, lead := range leads {
		if lead.Status == "qualified" {
			qualifiedLeads = append(qualifiedLeads, lead)
		}
	}
	
	if len(qualifiedLeads) > 0 {
		suggestions = append(suggestions, "\nQualified leads to consider:")
		for i, lead := range qualifiedLeads {
			if i >= 5 {
				break
			}
			suggestions = append(suggestions, fmt.Sprintf("- %s (Deadline: %s)", lead.Name, lead.Deadline))
		}
	}
	
	return suggestions
}

func parseDate(dateStr string) time.Time {
	t, _ := time.Parse("2006-01-02", dateStr)
	return t
}

func formatScheduleMessage(suggestions []string) string {
	if len(suggestions) == 0 {
		return "[ScholarshipOps Schedule] No upcoming applications this week."
	}
	
	msg := "[ScholarshipOps Schedule] Suggested applications for this week:\n"
	for _, s := range suggestions {
		msg += s + "\n"
	}
	return msg
}
