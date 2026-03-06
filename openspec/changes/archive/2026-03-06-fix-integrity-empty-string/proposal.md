## Why

Newer Unity versions return an empty string (`""`) for the `integrity` field in the live platform API instead of `null`. `ssri::Integrity::from_str("")` parses successfully and produces an `Integrity` with zero hashes. When `verify_checksum` later calls `i.check(...)` on that value, `ssri` panics with an index-out-of-bounds error because it assumes at least one hash is present. This crash prevents installation of any Unity version that returns an empty integrity string.

## What Changes

- Treat an empty `integrity` string as absent (`None`) in `deserialize_sri`, so that `verify_checksum` receives `None` and returns `CheckSumResult::NoCheckSum` instead of panicking

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `payload-format-detection`: the deserialization of the `integrity` field must now handle empty-string values in addition to `null` and valid SRI strings

## Impact

- **`uvm_live_platform/src/model/release.rs`**: one-line guard in `deserialize_sri` — skip `from_str` and return `Ok(None)` when the string is empty
- No behaviour change for `null` or valid integrity values; installations without a checksum continue to be skipped gracefully
