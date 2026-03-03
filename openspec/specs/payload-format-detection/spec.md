## ADDED Requirements

### Requirement: Detect gzipped tar payload format

The macOS PKG installer SHALL detect when a payload is a gzipped tar archive and extract it using tar.

#### Scenario: Payload is gzipped tar

- **WHEN** the payload file starts with gzip magic bytes (1f 8b) AND the decompressed content does NOT start with cpio magic (070707, 070701, 070702)
- **THEN** the installer SHALL extract the payload using `tar -zmxf`

### Requirement: Detect gzipped cpio payload format

The macOS PKG installer SHALL detect when a payload is a gzipped cpio archive and extract it using cpio.

#### Scenario: Payload is gzipped cpio ODC format

- **WHEN** the payload file starts with gzip magic bytes (1f 8b) AND the decompressed content starts with `070707` (cpio ODC magic)
- **THEN** the installer SHALL extract the payload using `gzip -dc <payload> | cpio -iu`

#### Scenario: Payload is gzipped cpio newc format

- **WHEN** the payload file starts with gzip magic bytes (1f 8b) AND the decompressed content starts with `070701` or `070702` (cpio newc magic)
- **THEN** the installer SHALL extract the payload using `gzip -dc <payload> | cpio -iu`

### Requirement: Extract to correct destination directory

The macOS PKG installer SHALL extract payload contents to the specified destination directory regardless of payload format.

#### Scenario: cpio extraction respects destination

- **WHEN** extracting a cpio payload to destination directory D
- **THEN** the cpio command SHALL run with `-D` or `--directory` set to D, OR the process SHALL change to directory D before extraction

#### Scenario: tar extraction respects destination

- **WHEN** extracting a tar payload to destination directory D
- **THEN** the tar command SHALL use `-C D` to extract to the destination

### Requirement: Backwards compatibility with tar payloads

The installer SHALL continue to successfully install Unity versions that use tar-format payloads.

#### Scenario: Install older Unity version with tar payload

- **WHEN** installing a Unity version that ships with a tar-format PKG payload
- **THEN** the installation SHALL complete successfully using tar extraction
