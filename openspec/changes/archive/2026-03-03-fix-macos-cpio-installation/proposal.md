## Why

Unity PKG installer payloads can be either gzipped tar or gzipped cpio archives. The macOS installer currently only supports tar format, causing installation failures for newer Unity versions (e.g., Unity 6000.3.10f1) that use cpio payloads.

## What Changes

- Add payload format detection to the macOS PKG installer
- Implement cpio extraction support for macOS (gzip -dc | cpio -iu pipeline)
- Maintain backwards compatibility with existing tar-based payloads
- Align macOS behavior with the Linux implementation which already handles both formats

## Capabilities

### New Capabilities

- `payload-format-detection`: Detect whether a PKG payload is tar or cpio format by examining magic bytes after decompression

### Modified Capabilities

- None (no existing spec-level requirements are changing, this is a bug fix to support an additional format)

## Impact

- **Code**: `uvm_install/src/sys/mac/pkg.rs` - the `tar()` and `untar()` methods need to detect format and dispatch to appropriate extractor
- **Dependencies**: Uses existing system commands (`gzip`, `cpio`, `tar`) - no new dependencies
- **Compatibility**: Fully backwards compatible - existing tar payloads continue to work, cpio payloads now also work
