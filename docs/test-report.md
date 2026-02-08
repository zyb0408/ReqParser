# Test Report

**Date**: 2026-02-08
**Project**: ReqParser (Tauri v2 + React + TypeScript)

## Summary

| Metric | Result |
|--------|--------|
| Total Rust tests | 102 |
| Tests passed | 102 |
| Tests failed | 0 |
| Pass rate | 100% |
| Cargo clippy warnings | 0 |
| Cargo build warnings | 0 |
| Frontend build (vite) | Pass |
| TypeScript type check (tsc --noEmit) | Pass |

## Test Distribution by Module

| Module | Test Count | Notes |
|--------|-----------|-------|
| `decoder.rs` | 52 (28 original + 24 new) | Recursive decoding: JWT, Base64, timestamp, URL encoding, JSON, compound |
| `curl_parser.rs` | 15 | cURL command parsing |
| `fetch_parser.rs` | 10 | fetch() call parsing |
| `parser.rs` | 10 | Raw HTTP text parsing |
| `detector.rs` | 13 | Input format detection |
| `models.rs` | 0 | Data structures only, covered indirectly |
| `error.rs` | 0 | Error types only, covered indirectly |
| `clipboard.rs` | 0 | macOS clipboard watcher, requires runtime environment |
| `lib.rs` | 0 | Tauri command handlers, requires Tauri runtime |

## New Tests Added (24 tests)

### Boundary tests (16)
- `test_decode_node_whitespace_only` - Whitespace-only input returns no decode
- `test_decode_node_with_leading_trailing_whitespace` - Trimming before decode
- `test_very_long_non_matching_string` - No panic on long inputs
- `test_timestamp_boundary_year_2000` - Smallest 10-digit timestamp in range
- `test_timestamp_boundary_year_2050` - Upper boundary timestamp
- `test_timestamp_just_below_range` - 9-digit value below range rejected
- `test_timestamp_just_above_range` - Above 2050 range rejected
- `test_timestamp_13_digit_boundary` - 13-digit millisecond boundary
- `test_base64_decode_with_unicode_result` - Chinese text in Base64
- `test_base64_exactly_20_chars` - Minimum Base64 length threshold
- `test_jwt_with_timestamps_in_payload` - JWT payload recursive timestamp decode
- `test_json_nested_object` - Nested JSON object handling
- `test_json_empty_object` / `test_json_empty_array` - Edge cases
- `test_compound_with_url_encoded_key` / `test_compound_with_empty_values` - Compound edge cases
- `test_url_decode_incomplete_percent` / `test_url_decode_empty_string` / `test_url_decode_percent_at_end` - URL decode edge cases

### Integration tests (5)
- `test_real_curl_example_end_to_end` - Real cURL from `real_example.md` with cookie/timestamp decode
- `test_real_fetch_example_end_to_end` - Real fetch with cookie children processing
- `test_full_pipeline_curl_parse_and_decode` - cURL parse -> recursive decode -> verify JWT/timestamp/cookie
- `test_full_pipeline_fetch_parse_and_decode` - fetch parse -> recursive decode -> verify timestamp/cookie
- `test_full_pipeline_raw_http_parse_and_decode` - Raw HTTP parse -> recursive decode -> verify all decoders

## Issues Found and Fixed

### 1. Clippy warning: `enum_variant_names` (fixed)
- **File**: `src-tauri/src/error.rs`
- **Issue**: All variants of `AppError` had the same `Error` suffix (`ParseError`, `ClipboardError`, `InternalError`)
- **Fix**: Added `#[allow(clippy::enum_variant_names)]` attribute since the naming is intentional and consistent with Rust error type conventions

## Coverage Analysis

### Well-covered paths
- JWT decoding with recursive child processing (header + payload extraction, nested timestamps)
- Timestamp decoding (10-digit, 13-digit, range validation)
- Base64 decoding (standard encoding, UTF-8 validation, length threshold)
- URL encoding/decoding (percent encoding, plus sign, Chinese characters)
- JSON parsing (objects, arrays, nested structures)
- Compound value parsing (k=v&k=v with recursive child decode)
- Full pipeline: parse -> detect -> decode for all 3 input formats (cURL, fetch, raw HTTP)

### Paths lacking test coverage
- `clipboard.rs`: Clipboard watcher requires macOS runtime and cannot be unit tested easily. Requires manual or integration testing.
- `lib.rs`: Tauri command handlers (`parse_text`, `check_http_like`, `toggle_clipboard_watcher`, `get_clipboard_watcher_status`) require Tauri runtime context. Could benefit from extracting logic to testable functions.
- `decoder.rs` body-level parsing: The `apply_recursive_decode` function has a body JSON check path (lines 41-49) that currently does nothing - this is a no-op placeholder for future functionality.
- Frontend components: No automated component tests. The TypeScript type check confirms type safety but not runtime behavior.

## Recommendations

1. Consider adding `#[cfg(test)] mod integration_tests` at the crate level for cross-module tests
2. The timestamp regex only matches exactly 10 or 13 digits - values like `946684800` (9 digits, year 2000) cannot be detected as timestamps. This is by design but worth documenting.
3. The clipboard watcher should be tested manually on each target platform
4. Frontend should add Vitest or React Testing Library tests for component logic
