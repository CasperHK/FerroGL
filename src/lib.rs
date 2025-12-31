#![no_std]

//! FerroGL: High-Performance, Safe Embedded UI in Rust
//!
//! ## Features
//! - 100% safe Rust, no C dependencies
//! - Zero-cost abstractions, no dynamic allocation
//! - Modern, Flexbox-inspired layout engine
//! - Hardware-agnostic driver interface (RISC-V, ARM, MCP)
//! - Designed for embedded and bare-metal targets
//!
//! ## Example
//! ```rust
//! use ferrogl::layout::{Layout, Direction, Align, Rect, LayoutChild};
//! let mut layout = Layout::new(Direction::Row, Align::Start, Rect { x: 0, y: 0, width: 320, height: 240 });
//! layout.add_child(LayoutChild { flex: 1, min_size: 10, max_size: 100, rect: Rect { x: 0, y: 0, width: 10, height: 240 } }).unwrap();
//! layout.compute();
//! ```

extern crate heapless;

pub mod layout;
pub mod widget;
pub mod driver;
pub mod state;

// Re-export key types for convenience
pub use layout::{Layout, Direction, Align, Rect, LayoutChild};
pub use driver::{DisplayDriver, DmaCapable, DisplayInfo, CpuArch};
pub use state::State;
