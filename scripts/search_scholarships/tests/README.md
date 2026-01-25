# ScholarshipOps Testing Guide

## Overview

This directory contains tests for the ScholarshipOps pipeline, including unit tests, integration tests, and test fixtures.

## Test Structure

```
tests/
├── fixtures/              # HTML fixture pages for testing
│   ├── taiwan_excluded.html
│   ├── home_fee_only.html
│   ├── phd_only.html
│   ├── disability_only.html
│   └── international_eligible.html
├── fixtures.json          # JSON test cases for triage rules
└── integration_test.rs    # Integration tests for parser and triage
```

## Running Tests

### Unit Tests

Run all unit tests (including module-level tests):

```bash
cd scripts/search_scholarships
cargo test
```

### Integration Tests

Run integration tests (marked with `#[ignore]`):

```bash
cd scripts/search_scholarships
ROOT=../.. cargo test -- --ignored
```

Note: Integration tests require the `ROOT` environment variable to point to the repository root (where `Config/rules.yaml` is located).

### Specific Test Cases

Run a specific test:

```bash
cargo test test_taiwan_excluded_hard_reject
```

Run tests matching a pattern:

```bash
cargo test test_hard_reject
```

## Test Fixtures

### HTML Fixtures

Located in `tests/fixtures/`, these HTML pages simulate real scholarship pages for testing:

- **taiwan_excluded.html**: Commonwealth scholarship that explicitly excludes Taiwan
- **home_fee_only.html**: UK domestic scholarship requiring Home fee status
- **phd_only.html**: PhD-only fellowship (should be rejected for master's applicants)
- **disability_only.html**: Disability support scholarship requiring certificate
- **international_eligible.html**: Glasgow scholarship open to international students

### JSON Fixtures

`tests/fixtures.json` contains structured test cases with expected bucket assignments and rule matches.

## Test Coverage

### Parser Tests

- Country eligibility parsing (`parse_eligible_countries`)
- Deadline normalization (`update_structured_dates`)
- URL canonicalization
- Deduplication key generation

### Triage Tests

- Hard reject rules (Taiwan excluded, Home fee only, PhD only, Disability only)
- Bucket assignment (A/B/C/X)
- Rule matching and scoring
- Confidence calculation

### Integration Tests

- End-to-end pipeline smoke test
- Multiple leads processing
- Bucket distribution validation

## Adding New Tests

### Adding a New HTML Fixture

1. Create HTML file in `tests/fixtures/`
2. Include realistic scholarship page structure
3. Add test case in `integration_test.rs`:

```rust
#[test]
#[ignore]
fn test_my_new_scenario() {
    let html = fs::read_to_string("tests/fixtures/my_fixture.html")
        .expect("Failed to read fixture");
    // ... test logic
}
```

### Adding a New JSON Test Case

Add to `tests/fixtures.json`:

```json
{
  "id": "TC-XXX",
  "name": "Test Case Name",
  "description": "What this tests",
  "input": { /* lead data */ },
  "expected": { /* expected results */ }
}
```

## CI Integration

Tests are automatically run in GitHub Actions:

- Unit tests: Run on every push
- Integration tests: Run with `cargo test -- --ignored` in CI
- Schema validation: Validates JSON schemas for agent outputs

## Troubleshooting

### Tests Fail with "Failed to load rules"

Ensure `ROOT` environment variable points to repository root:

```bash
export ROOT=/path/to/Smart_Zone
cargo test -- --ignored
```

### Tests Fail with "Failed to read fixture"

Ensure you're running tests from the `scripts/search_scholarships` directory:

```bash
cd scripts/search_scholarships
cargo test
```

### Library Not Found Errors

Ensure `Cargo.toml` includes `[lib]` section:

```toml
[lib]
name = "search_scholarships"
path = "src/lib.rs"
```
