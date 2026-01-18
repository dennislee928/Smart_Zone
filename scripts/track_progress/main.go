package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"time"

	"track_progress/notify"
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

type Deadline struct {
	Name string `yaml:"name"`
	Date string `yaml:"date"`
}

type DeadlinesFile struct {
	Deadlines []Deadline `yaml:"deadlines"`
}

type Statistics struct {
	Total           int
	InProgress      int
	Completed       int
	NotStarted      int
	Upcoming        []Application
	Upcoming7       int
	Upcoming14      int
	Upcoming21      int
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

	// Load deadlines
	deadlinesFile, err := loadDeadlines(root)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading deadlines: %v\n", err)
	}

	// Calculate statistics
	stats := calculateStatistics(appsFile.Applications, deadlinesFile)

	// Generate report
	report := generateReport(stats)

	// Send notifications
	if err := notify.SendNotifications(report); err != nil {
		fmt.Fprintf(os.Stderr, "Error sending notifications: %v\n", err)
	}

	fmt.Println(report)
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

func loadDeadlines(root string) (*DeadlinesFile, error) {
	yamlPath := filepath.Join(root, "tasks", "deadlines.yml")
	
	data, err := ioutil.ReadFile(yamlPath)
	if err != nil {
		return &DeadlinesFile{Deadlines: []Deadline{}}, nil
	}

	var deadlines DeadlinesFile
	if err := yaml.Unmarshal(data, &deadlines); err != nil {
		return &DeadlinesFile{Deadlines: []Deadline{}}, nil
	}

	return &deadlines, nil
}

func calculateStatistics(apps []Application, deadlines *DeadlinesFile) Statistics {
	stats := Statistics{
		Upcoming: []Application{},
	}
	
	now := time.Now()
	
	for _, app := range apps {
		stats.Total++
		
		switch app.Status {
		case "in_progress":
			stats.InProgress++
		case "submitted", "accepted", "rejected":
			stats.Completed++
		case "not_started":
			stats.NotStarted++
		}
		
		// Check upcoming deadlines
		if app.Deadline != "" {
			deadline, err := time.Parse("2006-01-02", app.Deadline)
			if err == nil && deadline.After(now) {
				daysUntil := int(time.Until(deadline).Hours() / 24)
				
				if daysUntil <= 7 {
					stats.Upcoming7++
					stats.Upcoming = append(stats.Upcoming, app)
				} else if daysUntil <= 14 {
					stats.Upcoming14++
				} else if daysUntil <= 21 {
					stats.Upcoming21++
				}
			}
		}
	}
	
	return stats
}

func generateReport(stats Statistics) string {
	report := "[ScholarshipOps Progress Report]\n\n"
	report += fmt.Sprintf("📊 Statistics:\n")
	report += fmt.Sprintf("- Total applications: %d\n", stats.Total)
	report += fmt.Sprintf("- In progress: %d\n", stats.InProgress)
	report += fmt.Sprintf("- Completed: %d\n", stats.Completed)
	report += fmt.Sprintf("- Not started: %d\n", stats.NotStarted)
	report += "\n"
	report += fmt.Sprintf("⏰ Upcoming deadlines:\n")
	report += fmt.Sprintf("- D-7: %d applications\n", stats.Upcoming7)
	report += fmt.Sprintf("- D-14: %d applications\n", stats.Upcoming14)
	report += fmt.Sprintf("- D-21: %d applications\n", stats.Upcoming21)
	
	if len(stats.Upcoming) > 0 {
		report += "\n🚨 Urgent (D-7 or less):\n"
		for _, app := range stats.Upcoming {
			report += fmt.Sprintf("- %s (Deadline: %s)\n", app.Name, app.Deadline)
		}
	}
	
	return report
}
