## Requirements

### Requirement: Empty integrity string is treated as absent
The `integrity` field deserializer SHALL treat an empty string (`""`) as equivalent to `null`, producing `None` rather than a parsed `Integrity` value with zero hashes.

#### Scenario: Empty string produces None
- **WHEN** the live platform API returns `"integrity": ""`
- **THEN** the deserialized `integrity` field SHALL be `None`

#### Scenario: Checksum verification is skipped for empty integrity
- **WHEN** the `integrity` field deserializes to `None` due to an empty string
- **THEN** `verify_checksum` SHALL return `CheckSumResult::NoCheckSum` without panicking

#### Scenario: Null integrity still produces None
- **WHEN** the live platform API returns `"integrity": null`
- **THEN** the deserialized `integrity` field SHALL be `None` (existing behaviour preserved)

#### Scenario: Valid integrity string still parses correctly
- **WHEN** the live platform API returns a valid SRI string (e.g. `"sha256-abc123..."`)
- **THEN** the deserialized `integrity` field SHALL be `Some(Integrity)` (existing behaviour preserved)
