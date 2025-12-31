//! Driver interface: Hardware-agnostic, DMA-ready

pub trait DisplayDriver {
    fn flush(&mut self, buffer: &[u8]);
    // ...more methods to be added...
}
