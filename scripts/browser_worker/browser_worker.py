#!/usr/bin/env python3
"""
Browser Worker - Playwright-based extraction for JS-heavy pages

Reads URLs from browser_queue.jsonl, renders with Playwright,
extracts scholarship data, detects API endpoints, and writes results.
"""

import json
import sys
import argparse
import asyncio
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional, Any
from playwright.async_api import async_playwright, Page, Browser, BrowserContext
import re


class ScholarshipExtractor:
    """Extract scholarship data from rendered page"""
    
    def __init__(self, page: Page):
        self.page = page
    
    async def extract(self, url: str) -> Dict[str, Any]:
        """Extract scholarship data from current page"""
        leads = []
        
        # Wait for page to load
        await self.page.wait_for_load_state("networkidle", timeout=10000)
        
        # Try multiple extraction strategies
        # Strategy 1: Look for common scholarship selectors
        scholarship_data = await self._extract_with_selectors()
        
        if scholarship_data:
            leads.append(scholarship_data)
        else:
            # Strategy 2: Look for structured data (JSON-LD)
            scholarship_data = await self._extract_from_json_ld()
            if scholarship_data:
                leads.append(scholarship_data)
        
        return {
            "leads": leads,
            "extraction_method": "playwright-selector" if leads else "failed"
        }
    
    async def _extract_with_selectors(self) -> Optional[Dict[str, Any]]:
        """Extract using CSS selectors"""
        try:
            # Common selectors for scholarship pages
            selectors = {
                "name": [
                    "h1.scholarship-title",
                    "h1",
                    ".scholarship-name",
                    "[data-scholarship-name]",
                ],
                "amount": [
                    ".scholarship-amount",
                    ".amount",
                    "[data-amount]",
                    "strong:contains('£')",
                    "strong:contains('$')",
                ],
                "deadline": [
                    ".deadline",
                    ".scholarship-deadline",
                    "[data-deadline]",
                    "time[datetime]",
                ],
                "eligibility": [
                    ".eligibility",
                    ".requirements",
                    "ul.eligibility-list",
                ],
            }
            
            result = {}
            evidence = []
            
            # Extract name
            for selector in selectors["name"]:
                try:
                    element = await self.page.query_selector(selector)
                    if element:
                        name = await element.inner_text()
                        if name and len(name.strip()) > 5:
                            result["name"] = name.strip()
                            xpath = await self.page.evaluate(
                                f"document.evaluate('//*[text()={json.dumps(name.strip())}]', document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue"
                            )
                            evidence.append({
                                "attribute": "name",
                                "snippet": name.strip(),
                                "selector": selector,
                                "xpath": None,  # XPath extraction needs more work
                                "method": "playwright-selector"
                            })
                            break
                except:
                    continue
            
            # Extract amount
            for selector in selectors["amount"]:
                try:
                    element = await self.page.query_selector(selector)
                    if element:
                        amount = await element.inner_text()
                        if amount and ("£" in amount or "$" in amount or "€" in amount):
                            result["amount"] = amount.strip()
                            evidence.append({
                                "attribute": "amount",
                                "snippet": amount.strip(),
                                "selector": selector,
                                "xpath": None,
                                "method": "playwright-selector"
                            })
                            break
                except:
                    continue
            
            # Extract deadline
            for selector in selectors["deadline"]:
                try:
                    element = await self.page.query_selector(selector)
                    if element:
                        deadline = await element.inner_text()
                        if deadline:
                            result["deadline"] = deadline.strip()
                            evidence.append({
                                "attribute": "deadline",
                                "snippet": deadline.strip(),
                                "selector": selector,
                                "xpath": None,
                                "method": "playwright-selector"
                            })
                            break
                except:
                    continue
            
            # Extract eligibility
            eligibility = []
            for selector in selectors["eligibility"]:
                try:
                    elements = await self.page.query_selector_all(selector + " li")
                    for el in elements:
                        text = await el.inner_text()
                        if text:
                            eligibility.append(text.strip())
                    if eligibility:
                        break
                except:
                    continue
            
            if eligibility:
                result["eligibility"] = eligibility
            
            if result.get("name"):
                result["extraction_evidence"] = evidence
                return result
            
        except Exception as e:
            print(f"Error in selector extraction: {e}", file=sys.stderr)
        
        return None
    
    async def _extract_from_json_ld(self) -> Optional[Dict[str, Any]]:
        """Extract from JSON-LD structured data"""
        try:
            json_ld_scripts = await self.page.query_selector_all('script[type="application/ld+json"]')
            
            for script in json_ld_scripts:
                content = await script.inner_text()
                try:
                    data = json.loads(content)
                    if isinstance(data, dict):
                        # Look for Scholarship type
                        if data.get("@type") == "Scholarship" or "Scholarship" in str(data):
                            result = {
                                "name": data.get("name", ""),
                                "amount": data.get("monetaryAmount", {}).get("value", "") if isinstance(data.get("monetaryAmount"), dict) else "",
                                "deadline": data.get("applicationDeadline", ""),
                                "eligibility": data.get("eligibleRegion", []),
                                "extraction_evidence": [{
                                    "attribute": "structured_data",
                                    "snippet": json.dumps(data, indent=2)[:500],
                                    "selector": "script[type='application/ld+json']",
                                    "xpath": None,
                                    "method": "json-ld"
                                }]
                            }
                            if result["name"]:
                                return result
                except json.JSONDecodeError:
                    continue
        
        except Exception as e:
            print(f"Error in JSON-LD extraction: {e}", file=sys.stderr)
        
        return None


class ApiDetector:
    """Detect API endpoints from network requests"""
    
    def __init__(self, page: Page):
        self.page = page
        self.api_endpoints = []
    
    async def detect(self) -> List[Dict[str, Any]]:
        """Detect API endpoints from network log"""
        # Intercept network requests
        api_patterns = [
            r"/api/[^/]+",
            r"/graphql",
            r"/rest/[^/]+",
            r"/v\d+/[^/]+",
        ]
        
        detected = []
        
        # Get all network requests
        # Note: This requires capturing requests during page load
        # For now, we'll scan the page source for API calls
        
        try:
            page_content = await self.page.content()
            
            # Look for fetch/axios calls in JavaScript
            js_patterns = [
                r"fetch\(['\"]([^'\"]+)['\"]",
                r"axios\.get\(['\"]([^'\"]+)['\"]",
                r"\$\.ajax\([^,]*url:\s*['\"]([^'\"]+)['\"]",
            ]
            
            for pattern in js_patterns:
                matches = re.finditer(pattern, page_content)
                for match in matches:
                    endpoint = match.group(1)
                    if any(re.search(pat, endpoint) for pat in api_patterns):
                        detected.append({
                            "url": endpoint,
                            "method": "GET",
                            "response_type": "json",
                            "sample_response": None
                        })
            
            # Look for API endpoints in page source
            for pattern in api_patterns:
                matches = re.finditer(pattern, page_content)
                for match in matches:
                    endpoint = match.group(0)
                    if endpoint not in [d["url"] for d in detected]:
                        detected.append({
                            "url": endpoint,
                            "method": "GET",
                            "response_type": "json",
                            "sample_response": None
                        })
        
        except Exception as e:
            print(f"Error detecting API endpoints: {e}", file=sys.stderr)
        
        return detected


async def process_url(
    browser: Browser,
    url: str,
    source_id: str,
    source_name: str,
    detection_reason: str,
    detected_api_endpoints: List[str],
) -> Dict[str, Any]:
    """Process a single URL with browser"""
    context = await browser.new_context(
        user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    )
    page = await context.new_page()
    
    try:
        # Navigate to page
        await page.goto(url, wait_until="networkidle", timeout=30000)
        
        # Extract scholarship data
        extractor = ScholarshipExtractor(page)
        extraction_result = await extractor.extract(url)
        
        # Detect API endpoints
        api_detector = ApiDetector(page)
        detected_apis = await api_detector.detect()
        
        # Merge with pre-detected endpoints
        all_endpoints = detected_apis + [
            {"url": ep, "method": "GET", "response_type": "json", "sample_response": None}
            for ep in detected_api_endpoints
        ]
        
        return {
            "url": url,
            "source_id": source_id,
            "status": "success" if extraction_result["leads"] else "no_data",
            "leads": extraction_result["leads"],
            "detected_api_endpoints": all_endpoints,
            "error": None,
            "processed_at": datetime.utcnow().isoformat() + "Z"
        }
    
    except Exception as e:
        return {
            "url": url,
            "source_id": source_id,
            "status": "error",
            "leads": [],
            "detected_api_endpoints": [],
            "error": str(e),
            "processed_at": datetime.utcnow().isoformat() + "Z"
        }
    
    finally:
        await context.close()


async def main():
    parser = argparse.ArgumentParser(description="Browser worker for JS-heavy pages")
    parser.add_argument("--worker-id", type=int, required=True, help="Worker ID (1-based)")
    parser.add_argument("--total-workers", type=int, required=True, help="Total number of workers")
    parser.add_argument("--queue-file", type=str, default="tracking/browser_queue.jsonl", help="Input queue file")
    parser.add_argument("--output-file", type=str, default="tracking/browser_results.jsonl", help="Output results file")
    parser.add_argument("--max-urls", type=int, default=50, help="Max URLs to process per worker")
    
    args = parser.parse_args()
    
    # Read queue file
    queue_path = Path(args.queue_file)
    if not queue_path.exists():
        print(f"Queue file not found: {queue_path}", file=sys.stderr)
        return
    
    entries = []
    with open(queue_path, "r") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                entry = json.loads(line)
                entries.append(entry)
            except json.JSONDecodeError:
                continue
    
    # Shard entries by worker ID
    worker_entries = [
        entry for i, entry in enumerate(entries)
        if (i % args.total_workers) == (args.worker_id - 1)
    ][:args.max_urls]
    
    if not worker_entries:
        print(f"No entries for worker {args.worker_id}", file=sys.stderr)
        return
    
    print(f"Worker {args.worker_id}: Processing {len(worker_entries)} URLs", file=sys.stderr)
    
    # Process URLs with Playwright
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        
        results = []
        for entry in worker_entries:
            result = await process_url(
                browser,
                entry["url"],
                entry["source_id"],
                entry.get("source_name", ""),
                entry.get("detection_reason", ""),
                entry.get("detected_api_endpoints", []),
            )
            results.append(result)
        
        await browser.close()
    
    # Write results
    output_path = Path(args.output_file)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    with open(output_path, "a") as f:  # Append mode for multiple workers
        for result in results:
            json.dump(result, f)
            f.write("\n")
    
    print(f"Worker {args.worker_id}: Completed {len(results)} URLs", file=sys.stderr)


if __name__ == "__main__":
    asyncio.run(main())
