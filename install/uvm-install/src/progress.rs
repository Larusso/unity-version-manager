use indicatif;
use std::ops::Deref;
use uvm_core::progress::ProgressHandler;

pub struct ProgressBar(indicatif::ProgressBar);
pub struct MultiProgress(indicatif::MultiProgress);

impl ProgressBar {
    pub fn new(len: u64) -> ProgressBar {
        ProgressBar(indicatif::ProgressBar::new(len))
    }
}

impl MultiProgress {
    pub fn new() -> MultiProgress {
        MultiProgress(indicatif::MultiProgress::new())
    }

    pub fn add(&self, pb:indicatif::ProgressBar) -> ProgressBar {
        ProgressBar(self.0.add(pb))
    }
}

impl Deref for ProgressBar {
    type Target = indicatif::ProgressBar;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for MultiProgress {
    type Target = indicatif::MultiProgress;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ProgressHandler for ProgressBar {
    fn inc(&self, delta: u64) {
        self.0.inc(delta)
    }

    fn set_length(&self, len: u64) {
        self.0.set_length(len);
        self.0.set_draw_delta(len/100);
    }

    fn set_position(&self, pos: u64) {
        self.0.set_position(pos)
    }

    fn finish(&self) {
        self.0.finish()
    }
}
