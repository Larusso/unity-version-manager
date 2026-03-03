## Context

The macOS PKG installer extracts Unity packages using a two-step process:
1. Extract the PKG container using `xar`
2. Extract the payload using `tar -zmxf` (assumes gzipped tar)

Unity has started shipping PKG files with cpio payloads instead of tar. The Linux implementation already handles both formats by using a `gzip | cpio` pipeline. The macOS implementation needs similar logic.

**Current code** (`uvm_install/src/sys/mac/pkg.rs`):
```rust
fn tar(&self, source: P, destination: D) -> InstallerResult<()> {
    Command::new("tar").arg("-zmxf").arg(source)...
}
```

**Linux code** (`uvm_install/src/sys/linux/pkg.rs`):
```rust
// Handles both Payload~ (raw cpio) and Payload (gzipped cpio)
gzip -dc Payload | cpio -iu
```

## Goals / Non-Goals

**Goals:**
- Support cpio-format payloads on macOS
- Maintain backwards compatibility with tar-format payloads
- Detect payload format automatically (no user configuration required)

**Non-Goals:**
- Changing the PKG extraction (xar) step - it works correctly
- Supporting other archive formats beyond tar and cpio
- Modifying the Linux implementation (already works)

## Decisions

### 1. Format Detection Strategy

**Decision**: Detect format by examining magic bytes after gzip decompression

**Rationale**:
- cpio archives start with `070707` (ODC) or `070701`/`070702` (newc)
- tar archives can be detected by `ustar` at offset 257, or by absence of cpio magic
- Checking first 6 bytes of decompressed stream is sufficient

**Alternatives considered**:
- Try tar first, fall back to cpio on failure: Wasteful for large files (5+ GB), poor UX
- Check file extension: Not reliable, both are named "Payload"
- Assume cpio always: Breaks backwards compatibility

### 2. Implementation Approach

**Decision**: Add a `detect_payload_format()` function, then dispatch to `extract_tar()` or `extract_cpio()`

**Rationale**:
- Clean separation of concerns
- Easy to test format detection independently
- Mirrors the structure in the Linux implementation

**Implementation outline**:
```rust
enum PayloadFormat { Tar, Cpio }

fn detect_payload_format(path: &Path) -> Result<PayloadFormat> {
    // Read first ~512 bytes, decompress, check magic
}

fn extract_payload(&self, payload: &Path, dest: &Path) -> Result<()> {
    match detect_payload_format(payload)? {
        PayloadFormat::Tar => self.extract_tar(payload, dest),
        PayloadFormat::Cpio => self.extract_cpio(payload, dest),
    }
}
```

### 3. cpio Extraction Method

**Decision**: Use `gzip -dc <payload> | cpio -iu` via shell pipeline

**Rationale**:
- Consistent with Linux implementation
- Uses standard macOS system tools (cpio is available by default)
- Handles large files efficiently via streaming

## Risks / Trade-offs

**[Risk]** cpio command not available on some macOS systems
→ **Mitigation**: cpio is part of macOS base system since at least 10.4. Not a realistic concern.

**[Risk]** Format detection reads beginning of large file
→ **Mitigation**: Only decompress first ~512 bytes for detection, not the entire file. Use `flate2` crate for in-process decompression to avoid spawning extra processes.

**[Trade-off]** Adding format detection adds complexity
→ **Accepted**: Necessary for backwards compatibility. The alternative (breaking old packages) is worse.
