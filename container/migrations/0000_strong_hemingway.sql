CREATE TABLE `applications` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`name` text NOT NULL,
	`deadline` text,
	`status` text DEFAULT 'not_started' NOT NULL,
	`current_stage` text,
	`next_action` text,
	`required_docs` text,
	`progress` integer DEFAULT 0,
	`notes` text,
	`created_at` integer,
	`updated_at` integer
);
--> statement-breakpoint
CREATE TABLE `criteria` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`criteria_json` text,
	`profile_json` text,
	`updated_at` integer
);
--> statement-breakpoint
CREATE TABLE `leads` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`name` text NOT NULL,
	`amount` text,
	`deadline` text,
	`source` text,
	`source_type` text,
	`status` text DEFAULT 'qualified' NOT NULL,
	`eligibility` text,
	`notes` text,
	`added_date` text,
	`url` text,
	`match_score` integer DEFAULT 0,
	`match_reasons` text,
	`hard_fail_reasons` text,
	`soft_flags` text,
	`bucket` text,
	`http_status` integer,
	`effort_score` integer,
	`trust_tier` text,
	`risk_flags` text,
	`matched_rule_ids` text,
	`eligible_countries` text,
	`is_taiwan_eligible` integer,
	`taiwan_eligibility_confidence` text,
	`deadline_date` text,
	`deadline_label` text,
	`intake_year` text,
	`study_start` text,
	`deadline_confidence` text,
	`canonical_url` text,
	`is_directory_page` integer DEFAULT false,
	`official_source_url` text,
	`source_domain` text,
	`confidence` integer,
	`eligibility_confidence` integer,
	`tags` text,
	`is_index_only` integer DEFAULT false,
	`first_seen_at` text,
	`last_checked_at` text,
	`next_check_at` text,
	`persistence_status` text,
	`source_seed` text,
	`check_count` integer DEFAULT 0,
	`created_at` integer,
	`updated_at` integer
);
--> statement-breakpoint
CREATE TABLE `source_health` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`url` text NOT NULL,
	`name` text NOT NULL,
	`source_type` text NOT NULL,
	`consecutive_failures` integer DEFAULT 0,
	`total_attempts` integer DEFAULT 0,
	`total_successes` integer DEFAULT 0,
	`last_status` text DEFAULT 'Unknown' NOT NULL,
	`last_http_code` integer,
	`last_error` text,
	`last_checked` text,
	`auto_disabled` integer DEFAULT false,
	`disabled_reason` text,
	`updated_at` integer
);
--> statement-breakpoint
CREATE UNIQUE INDEX `source_health_url_unique` ON `source_health` (`url`);--> statement-breakpoint
CREATE TABLE `sources` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`name` text NOT NULL,
	`type` text NOT NULL,
	`url` text NOT NULL,
	`enabled` integer DEFAULT true,
	`scraper` text,
	`priority` integer,
	`created_at` integer,
	`updated_at` integer
);
