# ScholarshipOps Agent Skills

This document describes the available Cursor agents and their scopes for the ScholarshipOps project.

## Overview

The ScholarshipOps system uses specialized agents to handle different aspects of scholarship discovery, extraction, and triage. Each agent is mapped to specific workstreams from the enhancement plan.

## Available Agents

### Config Auditor Agent
**Scope:** W0 (Preflight & Config Hygiene), W1 (Fix Scraper Type Mapping)
**Responsibilities:**
- Validate `sources.yml` schema and configuration
- Check for unsupported scraper types
- Verify required fields for each source type
- Detect misconfigured sources before pipeline execution
**Tools:**
- `validate_sources` binary
- YAML schema validation
- Source type enum checking

### Glasgow Discoverer Agent
**Scope:** W2 (Glasgow Coverage Expansion), W3 (Discovery Seed Sub-scraper)
**Responsibilities:**
- Sitemap-based discovery for `gla.ac.uk`
- Extract candidate URLs from discovery seed pages
- Implement controlled sub-scraper with domain allowlist/denylist
- Apply confidence scoring to candidate URLs
**Tools:**
- `discovery::discover_glasgow_sitemap`
- `discovery::discover_from_seed`
- `discovery::validate_candidate_heavy`

### Parsing Normalizer Agent
**Scope:** W4 (Parsing & Normalization)
**Responsibilities:**
- Deadline normalization (strict ISO/UK formats)
- Amount parsing and normalization
- Deduplication using content hashing
- URL canonicalization
**Tools:**
- `filter::update_structured_dates`
- `normalize::generate_entity_dedup_key`
- `normalize::canonicalize_candidate_url`

### Rules Engineer Agent
**Scope:** W5 (Hard-Reject Rule Hardening)
**Responsibilities:**
- Implement fee-status rules (`E-FEE-001`)
- Programme compatibility rules (`E-PROGRAM-001`, `E-PROGRAM-002`, `B-PROGRAM-UNK-001`)
- Disability eligibility rules (`E-DISABILITY-001`, `E-DISABILITY-002`)
- Update `Config/rules.yaml` with new rules
**Tools:**
- `rules::load_rules`
- `rules::apply_rules`
- Rule condition matching logic

### Trust & Fraud Agent
**Scope:** W6 (Fraud Detection & Trust Scoring)
**Responsibilities:**
- Fraud detection rules (`E-FRAUD-001`, `E-FRAUD-002`)
- Trust tier assignment and validation
- Source-level trust scoring
- Bucket threshold enforcement (e.g., Bucket A requires `min_trust_tier: "A"`)
**Tools:**
- `types::TrustTier` enum
- Trust tier mapping from source types
- Fraud pattern detection

### QA Agent
**Scope:** W7 (Tests & Fixtures)
**Responsibilities:**
- Create test fixtures (HTML pages for various scenarios)
- Unit tests for parser extraction
- Integration tests for triage bucket assignment
- E2E smoke tests
**Tools:**
- `tests/integration_test.rs`
- `tests/fixtures/` directory
- `cargo test -- --ignored` for e2e tests

## Agent Output Schemas

All agent outputs must conform to JSON schemas defined in `agent_skills/json_schemas/`:

- `candidate_url.schema.json` - Candidate URL discovery results
- `rule_change.schema.json` - Rule configuration changes
- `source_config.schema.json` - Source configuration validation results
- `triage_result.schema.json` - Triage bucket assignment results

## Usage

### Running Agents

Each agent can be invoked through Cursor's agent system:

1. **Config Auditor**: Run `validate_sources` binary before pipeline execution
2. **Glasgow Discoverer**: Automatically invoked in Stage 0.5 (Discovery)
3. **Parsing Normalizer**: Automatically invoked in Stage 0.6 (Candidate Normalization)
4. **Rules Engineer**: Manually invoked when updating rules.yaml
5. **Trust & Fraud**: Automatically invoked during triage (Stage 2)
6. **QA Agent**: Run `cargo test` for unit tests, `cargo test -- --ignored` for e2e

### Schema Validation

All agent outputs are validated against JSON schemas in CI:

```bash
# Validate candidate URLs
cat tracking/candidate_urls.jsonl | jq -s . | jsonschema candidate_url.schema.json

# Validate rule changes
cat Config/rules.yaml | yq . | jsonschema rule_change.schema.json
```

## Integration with Cursor Skills

If `.claude/skills` exists, agents can leverage:
- `systematic-debugging` - For troubleshooting pipeline failures
- `test-driven-development` - For creating test fixtures
- `writing-plans` - For documenting agent workflows

## Future Enhancements

- **Automated Agent Orchestration**: Chain agents together for end-to-end workflows
- **Agent Monitoring**: Track agent performance and success rates
- **Agent Learning**: Use historical data to improve agent heuristics
