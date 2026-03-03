## 1. Payload Format Detection

- [x] 1.1 Add `PayloadFormat` enum with `Tar` and `Cpio` variants to `uvm_install/src/sys/mac/pkg.rs`
- [x] 1.2 Implement `detect_payload_format()` function that reads first bytes of gzipped payload and checks for cpio magic (070707, 070701, 070702)
- [x] 1.3 Add `flate2` crate dependency to `uvm_install/Cargo.toml` for gzip decompression (if not already present)

## 2. cpio Extraction Implementation

- [x] 2.1 Implement `extract_cpio()` method using `gzip -dc <payload> | cpio -iu` pipeline
- [x] 2.2 Ensure cpio extraction runs in the correct destination directory (use `-D` flag or change working directory)

## 3. Refactor Extraction Flow

- [x] 3.1 Rename existing `tar()` method to `extract_tar()` for clarity
- [x] 3.2 Create new `extract_payload()` method that calls `detect_payload_format()` and dispatches to appropriate extractor
- [x] 3.3 Update `untar()` method to call `extract_payload()` instead of `tar()` directly

## 4. Testing

- [x] 4.1 Test installation with Unity 6000.3.10f1 (cpio payload) to verify fix works
- [x] 4.2 Test installation with an older Unity version (tar payload) to verify backwards compatibility
- [x] 4.3 Run `cargo test -p uvm_install` to ensure no regressions
