## 1. Extend ProgressHandler trait

- [x] 1.1 Add `set_message(&self, msg: &str)` default method to ProgressHandler trait in `uvm_install/src/install/loader.rs`
- [x] 1.2 Update ProgressHandler trait documentation with usage examples for different phases

## 2. Create progress handler implementation

- [x] 2.1 Create progress module in `uvm/src/commands/` with indicatif-based ProgressHandler implementation
- [x] 2.2 Implement wrapper struct around `indicatif::ProgressBar` that implements ProgressHandler trait
- [x] 2.3 Add TTY/pipe detection using `std::io::IsTerminal` on stdout and stderr, check CI env var
- [x] 2.4 Implement fallback SimpleProgressHandler for non-interactive environments using milestone messages
- [x] 2.5 Add helper functions for formatting bytes (KB/MB/GB) and time (seconds/minutes/hours)

## 3. Add multi-progress coordinator

- [x] 3.1 Create MultiProgressCoordinator struct using `indicatif::MultiProgress` for component hierarchy
- [x] 3.2 Implement overall progress bar showing total component count and completion
- [x] 3.3 Add method to create child progress bars for individual components
- [x] 3.4 Implement component status tracking (pending, downloading, installing, complete)
- [x] 3.5 Store MultiProgress as global/shared state for log integration access

## 4. Wire progress through InstallOptions

- [x] 4.1 Add `with_progress_handler` method to InstallOptions builder accepting trait object
- [x] 4.2 Store progress handler as `Option<Box<dyn ProgressHandler>>` in InstallOptions
- [x] 4.3 Pass progress handler through to Loader::set_progress_handle in install flow

## 5. Add progress to installation phases

- [x] 5.1 Add spinner for "Fetching Unity version metadata..." before fetch_release call in InstallOptions::install
- [x] 5.2 Add spinner for "Resolving component dependencies..." before graph construction
- [x] 5.3 Update component download loop to create individual progress bars per component
- [x] 5.4 Add component name and type (Editor/Module) to progress bar titles

## 6. Enhance Loader with download metrics

- [x] 6.1 Track download start time in Loader::download for speed calculation
- [x] 6.2 Update DownloadProgress to calculate and report download speed via set_message
- [x] 6.3 Add download completion message with final size and time via finish callback
- [x] 6.4 Handle resume scenarios by adjusting initial progress position

## 7. Add progress to platform installers

- [x] 7.1 Pass progress handler to create_installer functions in `uvm_install/src/install/mod.rs`
- [x] 7.2 Update platform installer structs to accept optional ProgressHandler in constructor
- [x] 7.3 Add spinner with "Extracting..." message in pkg/xz/zip installer before_install hooks
- [x] 7.4 Add spinner with "Installing..." message in installer install_handler implementations
- [x] 7.5 Call finish on progress handler after installation completes

## 8. Implement installation summary

- [x] 8.1 Track installation start time at beginning of install command execute()
- [x] 8.2 Track total bytes downloaded by summing Loader responses via coordinator
- [x] 8.3 Count total components installed via coordinator.components_installed()
- [x] 8.4 Display formatted summary in install.rs after successful installation with time, data, count, path

## 9. Integrate logging with indicatif

- [x] 9.1 Use indicatif_log_bridge::LogWrapper as the custom log writer
- [x] 9.2 Route log messages through MultiProgress::println() via LogWrapper
- [x] 9.3 Fall back to standard output when no progress bars (LogWrapper handles this)
- [x] 9.4 Update flexi_logger initialization to use LogWrapper in main.rs
- [ ] 9.5 Test verbose logging (debug/trace) doesn't corrupt progress display

## 10. Update CLI command integration

- [x] 10.1 Detect interactive mode using IsTerminal on stdout/stderr and CI env var
- [x] 10.2 Create MultiProgressCoordinator in install.rs::execute only when interactive
- [x] 10.3 Pass coordinator to InstallOptions via with_progress_handler
- [x] 10.4 Update error handling to clear progress display before showing errors
- [x] 10.5 Ensure final success message uses console::style as before

## 11. Testing and validation

- [ ] 11.1 Test single component installation (editor only) shows progress bars
- [ ] 11.2 Test multi-component installation (editor + 2-3 modules) shows hierarchy
- [ ] 11.3 Test stdout redirect: `uvm install ... > output.txt` shows milestone messages only
- [ ] 11.4 Test stderr redirect: `uvm install ... 2> errors.txt` disables progress bars
- [ ] 11.5 Test full pipe: `uvm install ... 2>&1 | tee log.txt` disables progress bars
- [ ] 11.6 Test with verbose logging: `uvm install ... -vv` shows logs above progress bars
- [ ] 11.7 Test error scenarios (network failure, disk full) clear progress cleanly
- [ ] 11.8 Verify progress works correctly on Windows, Linux, macOS terminals
- [ ] 11.9 Test CI environment (CI=true) disables progress bars
