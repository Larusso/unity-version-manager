use indicatif::{HumanBytes, MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use uvm_install::ProgressHandler;

static GLOBAL_MULTI_PROGRESS: OnceLock<MultiProgress> = OnceLock::new();

/// Returns the global MultiProgress instance, creating it on first call.
/// The same instance must be passed to LogWrapper so logging and progress bars
/// are coordinated (log messages suspend the bars instead of redrawing over them).
pub fn global_multi_progress() -> &'static MultiProgress {
    GLOBAL_MULTI_PROGRESS.get_or_init(MultiProgress::new)
}

/// Global progress configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressMode {
    Auto,     // Detect based on TTY
    Enabled,  // Force enabled via --progress
    Disabled, // Force disabled via --no-progress
}

static PROGRESS_MODE: Mutex<ProgressMode> = Mutex::new(ProgressMode::Auto);

/// Configure progress mode based on --progress/--no-progress flags
pub fn set_progress_mode(enabled: bool, disabled: bool) {
    let mut mode = PROGRESS_MODE.lock().unwrap();
    *mode = if enabled {
        ProgressMode::Enabled
    } else if disabled {
        ProgressMode::Disabled
    } else {
        ProgressMode::Auto
    };
}

/// Get the current progress mode
pub fn get_progress_mode() -> ProgressMode {
    *PROGRESS_MODE.lock().unwrap()
}

/// Detects if the current environment is interactive (has a TTY and not in CI).
pub fn is_interactive() -> bool {
    use std::io::IsTerminal;

    let mode = *PROGRESS_MODE.lock().unwrap();

    match mode {
        ProgressMode::Enabled => true,
        ProgressMode::Disabled => false,
        ProgressMode::Auto => {
            std::io::stdout().is_terminal()
                && std::io::stderr().is_terminal()
                && std::env::var("CI").is_err()
        }
    }
}

/// Progress handler that wraps an indicatif ProgressBar.
pub struct IndicatifProgressHandler {
    bar: ProgressBar,
    start_time: Instant,
    component_name: String,
}

impl IndicatifProgressHandler {
    pub fn new(bar: ProgressBar) -> Self {
        Self {
            bar,
            start_time: Instant::now(),
            component_name: String::new(),
        }
    }

    pub fn with_name(bar: ProgressBar, name: String) -> Self {
        Self {
            bar,
            start_time: Instant::now(),
            component_name: name,
        }
    }

    pub fn bar(&self) -> &ProgressBar {
        &self.bar
    }
}

impl ProgressHandler for IndicatifProgressHandler {
    fn finish(&self) {
        self.bar.finish_with_message(self.bar.message());
    }

    fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    fn set_length(&self, len: u64) {
        self.bar.set_length(len);
    }

    fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    fn begin_extraction_progress(&self, total_bytes: u64) {
        self.bar.reset();
        self.bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "  {{spinner}} {}: {{msg}} [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{bytes_per_sec}}, {{eta}})",
                    self.component_name
                ))
                .unwrap()
                .progress_chars("=>-"),
        );
        self.bar.enable_steady_tick(std::time::Duration::from_millis(100));
        self.bar.set_length(total_bytes);
        self.bar.set_message(format!(
            "Extracting {} files",
            HumanBytes(total_bytes)
        ));
    }

    fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());

        // Switch template based on state
        if msg.starts_with("Downloading") {
            // Switch to download progress bar style
            self.bar.set_style(
                ProgressStyle::default_bar()
                    .template(&format!("  {{spinner}} {}: {{msg}} [{{bar:40.green/blue}}] {{bytes}}/{{total_bytes}} ({{bytes_per_sec}}, {{eta}})", self.component_name))
                    .unwrap()
                    .progress_chars("=>-"),
            );
        } else if msg.starts_with("Installing") || msg.starts_with("Extracting") || msg.starts_with("Unpacking") {
            // Reset bar state to clear progress data, then switch to spinner-only style
            self.bar.reset();
            self.bar.set_style(
                ProgressStyle::default_spinner()
                    .template(&format!("  {{spinner}} {}: {{msg}}", self.component_name))
                    .unwrap(),
            );
            self.bar
                .enable_steady_tick(std::time::Duration::from_millis(100));
        } else if msg.starts_with("✓") || msg.starts_with("Waiting") {
            // Reset and switch to simple text-only style (no spinner animation for completed/waiting)
            self.bar.reset();
            self.bar.set_style(
                ProgressStyle::default_spinner()
                    .template(&format!("  {{spinner}} {}: {{msg}}", self.component_name))
                    .unwrap(),
            );
            // Disable spinning for static states
            self.bar.disable_steady_tick();
        }
    }
}

/// Simple progress handler for non-interactive environments.
/// Outputs milestone messages instead of progress bars.
pub struct SimpleProgressHandler {
    component_name: String,
    total_size: Option<u64>,
}

impl SimpleProgressHandler {
    pub fn new(component_name: String) -> Self {
        Self {
            component_name,
            total_size: None,
        }
    }
}

impl ProgressHandler for SimpleProgressHandler {
    fn finish(&self) {
        if let Some(size) = self.total_size {
            eprintln!(
                "✓ Downloaded {} ({})",
                self.component_name,
                HumanBytes(size)
            );
        } else {
            eprintln!("✓ Completed {}", self.component_name);
        }
    }

    fn inc(&self, _delta: u64) {
        // No-op for simple handler
    }

    fn set_length(&self, len: u64) {
        // Store for final message
        let handler = self as *const Self as *mut Self;
        unsafe {
            (*handler).total_size = Some(len);
        }
        eprintln!(
            "Downloading {} ({})...",
            self.component_name,
            HumanBytes(len)
        );
    }

    fn set_position(&self, _pos: u64) {
        // No-op for simple handler
    }

    fn set_message(&self, msg: &str) {
        eprintln!("{}: {}", self.component_name, msg);
    }

    fn begin_extraction_progress(&self, total_bytes: u64) {
        eprintln!(
            "{}: Extracting {} ({})...",
            self.component_name,
            self.component_name,
            HumanBytes(total_bytes)
        );
    }

    fn initialize_components(&self, components: &[(String, String)]) {
        eprintln!("Installing {} components...", components.len());
    }

    fn get_component_handler(&self, component_id: &str) -> Option<Box<dyn ProgressHandler>> {
        // Return a new simple handler for this component
        Some(Box::new(SimpleProgressHandler::new(
            component_id.to_string(),
        )))
    }
}

/// Coordinates multiple progress bars for component installation hierarchy.
pub struct MultiProgressCoordinator {
    multi: Arc<MultiProgress>,
    overall_bar: ProgressBar,
    phase_spinner: Mutex<Option<ProgressBar>>,
    completed_components: Arc<Mutex<usize>>,
    component_handlers:
        Arc<Mutex<std::collections::HashMap<String, Arc<IndicatifProgressHandler>>>>,
    total_bytes_downloaded: Arc<Mutex<u64>>,
}

impl MultiProgressCoordinator {
    /// Create a new multi-progress coordinator.
    pub fn new(total_components: usize) -> Self {
        let multi = Arc::new((*global_multi_progress()).clone());

        let overall_bar = multi.add(ProgressBar::new(total_components as u64));
        overall_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} components")
                .unwrap()
                .progress_chars("=>-"),
        );
        overall_bar.enable_steady_tick(std::time::Duration::from_millis(100));

        Self {
            multi,
            overall_bar,
            phase_spinner: Mutex::new(None),
            completed_components: Arc::new(Mutex::new(0)),
            component_handlers: Arc::new(Mutex::new(std::collections::HashMap::new())),
            total_bytes_downloaded: Arc::new(Mutex::new(0)),
        }
    }

    /// Show a phase-level status message (e.g. "Fetching metadata...", "Resolving dependencies...").
    /// Creates a spinner on first call; updates the message on subsequent calls.
    /// The spinner is cleared when initialize_components is called.
    fn set_phase_message(&self, msg: &str) {
        let mut spinner_lock = self.phase_spinner.lock().unwrap();
        if let Some(ref spinner) = *spinner_lock {
            spinner.set_message(msg.to_string());
        } else {
            let spinner = self.multi.insert_before(&self.overall_bar, ProgressBar::new_spinner());
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner} {msg}")
                    .unwrap(),
            );
            spinner.set_message(msg.to_string());
            spinner.enable_steady_tick(std::time::Duration::from_millis(100));
            *spinner_lock = Some(spinner);
        }
    }

    /// Pre-create progress bars for all components that will be installed.
    pub fn init_component_progress(&self, components: &[(String, String)]) {
        log::debug!(
            "initialize_components called with {} components",
            components.len()
        );
        let mut handlers = self.component_handlers.lock().unwrap();

        for (component_id, component_type) in components {
            log::debug!(
                "Creating progress bar for {} ({})",
                component_id,
                component_type
            );
            let display_name = if component_type == "Editor" {
                format!("Unity Editor")
            } else {
                component_id.clone()
            };

            let handler = self.create_download_progress(&display_name);

            // Set initial "Waiting..." state with spinner-only template
            handler.bar.set_style(
                ProgressStyle::default_spinner()
                    .template(&format!("  {{spinner}} {}: {{msg}}", display_name))
                    .unwrap(),
            );
            handler.bar.set_message("Waiting...");

            let prev = handlers.insert(component_id.clone(), Arc::new(handler));
            if prev.is_some() {
                log::warn!("Duplicate component: {}", component_id);
            }
        }

        log::debug!("Stored {} handlers in HashMap", handlers.len());

        // Clear the phase spinner now that component bars are visible
        let mut spinner_lock = self.phase_spinner.lock().unwrap();
        if let Some(spinner) = spinner_lock.take() {
            spinner.finish_and_clear();
        }

        // Update total count
        self.overall_bar.set_length(components.len() as u64);
    }

    /// Get the progress handler for a specific component.
    pub fn get_component_handler(
        &self,
        component_id: &str,
    ) -> Option<Arc<IndicatifProgressHandler>> {
        let handlers = self.component_handlers.lock().unwrap();
        handlers.get(component_id).cloned()
    }

    /// Get a reference to the MultiProgress instance for logging integration.
    pub fn multi(&self) -> Arc<MultiProgress> {
        self.multi.clone()
    }

    /// Create a progress bar for a component download.
    pub fn create_download_progress(&self, component_name: &str) -> IndicatifProgressHandler {
        let bar = self.multi.add(ProgressBar::new(0));
        bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!("  {{spinner}} {}: {{msg}} [{{bar:40.green/blue}}] {{bytes}}/{{total_bytes}} ({{bytes_per_sec}}, {{eta}})", component_name))
                .unwrap()
                .progress_chars("=>-"),
        );

        IndicatifProgressHandler::with_name(bar, component_name.to_string())
    }

    /// Create a spinner for a component installation phase.
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        let spinner = self.multi.add(ProgressBar::new_spinner());
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("  {spinner} {msg}")
                .unwrap(),
        );
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        spinner
    }

    /// Finish the overall progress display.
    pub fn finish(&self) {
        let completed = *self.completed_components.lock().unwrap();
        self.overall_bar
            .finish_with_message(format!("✓ {} components installed", completed));
    }

    /// Number of components that completed installation.
    pub fn components_installed(&self) -> usize {
        *self.completed_components.lock().unwrap()
    }

    /// Total bytes downloaded across all components.
    pub fn bytes_downloaded(&self) -> u64 {
        *self.total_bytes_downloaded.lock().unwrap()
    }

    /// Clear all progress bars (e.g., on error).
    pub fn clear(&self) {
        self.multi.clear().ok();
    }
}

impl ProgressHandler for MultiProgressCoordinator {
    fn finish(&self) {
        self.finish();
    }

    fn inc(&self, _delta: u64) {
        // Not used at coordinator level
    }

    fn set_length(&self, _len: u64) {
        // Not used at coordinator level
    }

    fn set_position(&self, _pos: u64) {
        // Not used at coordinator level
    }

    fn set_message(&self, msg: &str) {
        self.set_phase_message(msg);
    }

    fn create_child_handler(
        &self,
        component_name: &str,
        component_type: &str,
    ) -> Option<Box<dyn ProgressHandler>> {
        let display_name = if component_type == "Editor" {
            format!("Unity Editor ({})", component_name)
        } else {
            component_name.to_string()
        };

        Some(Box::new(self.create_download_progress(&display_name)))
    }

    fn mark_component_complete(&self) {
        let mut completed = self.completed_components.lock().unwrap();
        *completed += 1;
        self.overall_bar.set_position(*completed as u64);
    }

    fn set_total_components(&self, count: usize) {
        self.overall_bar.set_length(count as u64);
    }

    fn initialize_components(&self, components: &[(String, String)]) {
        self.init_component_progress(components);
    }

    fn get_component_handler(&self, component_id: &str) -> Option<Box<dyn ProgressHandler>> {
        let handlers = self.component_handlers.lock().unwrap();
        let handler = handlers.get(component_id)?;
        Some(Box::new(ArcProgressHandler(
            handler.clone(),
            self.total_bytes_downloaded.clone(),
        )))
    }
}

/// Wrapper to allow Arc<IndicatifProgressHandler> to be returned as Box<dyn ProgressHandler>.
/// Also accumulates bytes downloaded into the shared coordinator counter on finish().
struct ArcProgressHandler(Arc<IndicatifProgressHandler>, Arc<Mutex<u64>>);

/// Wrapper to allow Arc<MultiProgressCoordinator> to be passed as a ProgressHandler
pub struct ArcProgressCoordinator(pub Arc<MultiProgressCoordinator>);

impl ProgressHandler for ArcProgressHandler {
    fn finish(&self) {
        // Capture bytes before the bar is reset by the subsequent "Installing..." set_message call.
        let bytes = self.0.bar().position();
        *self.1.lock().unwrap() += bytes;
        self.0.finish();
    }

    fn inc(&self, delta: u64) {
        self.0.inc(delta);
    }

    fn set_length(&self, len: u64) {
        self.0.set_length(len);
    }

    fn set_position(&self, pos: u64) {
        self.0.set_position(pos);
    }

    fn set_message(&self, msg: &str) {
        self.0.set_message(msg);
    }

    fn begin_extraction_progress(&self, total_bytes: u64) {
        self.0.begin_extraction_progress(total_bytes);
    }
}

impl ProgressHandler for ArcProgressCoordinator {
    fn finish(&self) {
        self.0.finish();
    }

    fn inc(&self, delta: u64) {
        self.0.inc(delta);
    }

    fn set_length(&self, len: u64) {
        self.0.set_length(len);
    }

    fn set_position(&self, pos: u64) {
        self.0.set_position(pos);
    }

    fn set_message(&self, msg: &str) {
        self.0.set_message(msg);
    }

    fn begin_extraction_progress(&self, total_bytes: u64) {
        self.0.begin_extraction_progress(total_bytes);
    }

    fn create_child_handler(
        &self,
        component_name: &str,
        component_type: &str,
    ) -> Option<Box<dyn ProgressHandler>> {
        self.0.create_child_handler(component_name, component_type)
    }

    fn mark_component_complete(&self) {
        self.0.mark_component_complete();
    }

    fn set_total_components(&self, count: usize) {
        self.0.set_total_components(count);
    }

    fn initialize_components(&self, components: &[(String, String)]) {
        self.0.initialize_components(components);
    }

    fn get_component_handler(&self, component_id: &str) -> Option<Box<dyn ProgressHandler>> {
        let handlers = self.0.component_handlers.lock().unwrap();
        let handler = handlers.get(component_id)?;
        Some(Box::new(ArcProgressHandler(
            handler.clone(),
            self.0.total_bytes_downloaded.clone(),
        )))
    }
}

