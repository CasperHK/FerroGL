//! Driver interface: Hardware-agnostic, DMA-ready
//!
//! This module defines the traits and types required to integrate FerroGL with
//! various display hardware, supporting RISC-V, ARM, and MCP platforms.
//! The interface is designed to be extensible and efficient, with optional DMA support.

/// Supported CPU architectures for display drivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuArch {
    RiscV,
    Arm,
    Mcp,
    Unknown,
}

/// Display information for initialization and configuration.
pub struct DisplayInfo {
    pub width: u16,
    pub height: u16,
    pub color_depth: u8, // bits per pixel
    pub arch: CpuArch,
    // ...more fields as needed...
}

/// DMA transfer trait for displays supporting DMA.
pub trait DmaCapable {
    /// Initiate a DMA transfer of the framebuffer to the display.
    fn dma_transfer(&mut self, buffer: &[u8]);
}

/// The core display driver trait for FerroGL.
pub trait DisplayDriver {
    /// Initialize the display hardware.
    fn init(&mut self, info: &DisplayInfo);

    /// Flush a region of the framebuffer to the display.
    ///
    /// # Arguments
    /// * `x` - X coordinate of the region's top-left corner
    /// * `y` - Y coordinate of the region's top-left corner
    /// * `width` - Width of the region
    /// * `height` - Height of the region
    /// * `buffer` - Pixel data (format depends on color_depth)
    fn flush(&mut self, x: u16, y: u16, width: u16, height: u16, buffer: &[u8]);

    /// Optional: Enable or disable DMA if supported.
    fn enable_dma(&mut self, _enabled: bool) {
        // Default: do nothing
    }
}

/// Example: A simple framebuffer-based display driver for demonstration.
pub struct FramebufferDisplay<'a> {
    pub framebuffer: &'a mut [u8],
    pub info: DisplayInfo,
    pub dma_enabled: bool,
}

impl<'a> FramebufferDisplay<'a> {
    pub fn new(framebuffer: &'a mut [u8], info: DisplayInfo) -> Self {
        Self {
            framebuffer,
            info,
            dma_enabled: false,
        }
    }
}

impl<'a> DisplayDriver for FramebufferDisplay<'a> {
    fn init(&mut self, _info: &DisplayInfo) {
        // Initialize hardware, clear framebuffer, etc.
        for byte in self.framebuffer.iter_mut() {
            *byte = 0;
        }
    }

    fn flush(&mut self, x: u16, y: u16, width: u16, height: u16, buffer: &[u8]) {
        // For demonstration: copy buffer into framebuffer at the correct offset.
        // Real implementation would handle pixel format and bounds checking.
        let stride = self.info.width as usize;
        for row in 0..height as usize {
            let dst_start = ((y as usize + row) * stride) + x as usize;
            let src_start = row * width as usize;
            let dst_end = dst_start + width as usize;
            let src_end = src_start + width as usize;
            if dst_end <= self.framebuffer.len() && src_end <= buffer.len() {
                self.framebuffer[dst_start..dst_end]
                    .copy_from_slice(&buffer[src_start..src_end]);
            }
        }
        // Optionally trigger DMA if enabled
        if self.dma_enabled {
            self.dma_transfer(self.framebuffer);
        }
    }

    fn enable_dma(&mut self, enabled: bool) {
        self.dma_enabled = enabled;
    }
}

impl<'a> DmaCapable for FramebufferDisplay<'a> {
    fn dma_transfer(&mut self, buffer: &[u8]) {
        // Stub: In real hardware, trigger DMA transfer here.
        // For now, just a placeholder.
        let _ = buffer;
        // e.g., start_dma_transfer(buffer);
    }
}
