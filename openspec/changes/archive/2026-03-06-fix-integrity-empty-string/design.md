## Context

`ssri::Integrity::from_str("")` does not return an error — it silently produces an `Integrity` value with an empty hash list. The existing `deserialize_sri` function only guards against `None` and parse errors, so an empty string passes through as `Some(Integrity{hashes: []})`. When `verify_checksum` calls `i.check(...)` on that value, `ssri` panics with an index-out-of-bounds.

The affected code path:
1. Live platform API returns `"integrity": ""` for newer Unity versions
2. `deserialize_sri` calls `Integrity::from_str("")` → succeeds, zero hashes
3. `verify_checksum` receives `Some(empty_integrity)`, calls `i.check(&bytes)` → panic

## Goals / Non-Goals

**Goals:**
- Treat `""` as absent integrity (same as `null`) in `deserialize_sri`

**Non-Goals:**
- Changing checksum verification logic
- Handling other malformed integrity values differently than today

## Decisions

### Guard empty string before `from_str`

Add `if s.is_empty() { return Ok(None); }` in `deserialize_sri` before the `Integrity::from_str` call. This is the earliest, safest point to intercept the bad value — it requires no changes to `verify_checksum` or callers, and keeps the existing `Err(_) => Ok(None)` fallback for other malformed values.

**Alternative considered**: Validate the parsed `Integrity` after construction (e.g., check `hashes.is_empty()`). Rejected — requires knowledge of `ssri` internals and is more fragile than rejecting the empty input directly.
