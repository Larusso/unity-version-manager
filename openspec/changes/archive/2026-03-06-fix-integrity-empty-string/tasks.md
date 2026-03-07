## 1. Fix

- [x] 1.1 In `uvm_live_platform/src/model/release.rs`, add an early return in `deserialize_sri`: if the deserialized string is empty, return `Ok(None)` before calling `Integrity::from_str`

## 2. Tests

- [x] 2.1 Add a `#[cfg(test)]` module to `uvm_live_platform/src/model/release.rs` with unit tests for `deserialize_sri`:
  - `empty_string_integrity_deserializes_to_none` — deserialize `{"integrity": ""}` into `UnityReleaseFile`, assert `integrity` is `None`
  - `null_integrity_deserializes_to_none` — deserialize `{"integrity": null}`, assert `integrity` is `None`
  - `missing_integrity_field_deserializes_to_none` — deserialize `{}` (field absent), assert `integrity` is `None`
  - `valid_integrity_deserializes_to_some` — deserialize a valid SRI string, assert `integrity` is `Some`

## 3. Verify

- [x] 3.1 Run `cargo test -p uvm_live_platform` and confirm all tests pass
- [x] 3.2 Run `cargo clippy -p uvm_live_platform` and confirm no new warnings
