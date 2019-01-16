
pub trait ProgressHandler {
    fn finish(&self);
    fn inc(&self, delta: u64);
    fn set_length(&self, len: u64);
    fn set_position(&self, pos: u64);
}
