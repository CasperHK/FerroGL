#![cfg_attr(not(test), no_std)]

//! FerroGL: High-Performance, Safe Embedded UI in Rust
//!
//! A complete Rust rewrite of LVGL (Light and Versatile Graphics Library) modules,
//! designed for embedded and bare-metal targets.
//!
//! ## Features
//! - 100% safe Rust, no C dependencies
//! - Zero-cost abstractions, no dynamic allocation
//! - Modern, Flexbox-inspired layout engine + Grid layout
//! - Hardware-agnostic driver interface (RISC-V, ARM, MCP)
//! - Full widget set: Button, Label, Slider, Checkbox, Arc, Bar, Switch
//! - Drawing primitives: lines, rectangles, circles, arcs
//! - Animation engine with easing functions
//! - Reactive state management
//! - Event system with typed events
//! - Input device abstraction (touch, keyboard, encoder)
//! - Bitmap font system
//! - Style system with colors, borders, padding, opacity
//!
//! ## Example
//! ```rust
//! use ferrogl::layout::{Layout, Direction, Align, Rect, LayoutChild};
//! let mut layout = Layout::new(Direction::Row, Align::Start, Rect { x: 0, y: 0, width: 320, height: 240 });
//! layout.add_child(LayoutChild { flex: 1, min_size: 10, max_size: 100, rect: Rect { x: 0, y: 0, width: 10, height: 240 } }).unwrap();
//! layout.compute();
//! ```

pub mod animation;
pub mod color;
pub mod draw;
pub mod driver;
pub mod event;
pub mod font;
pub mod input;
pub mod layout;
pub mod state;
pub mod style;
pub mod widget;

// Re-export key types for convenience
pub use animation::{Animation, easing};
pub use color::Color;
pub use driver::{CpuArch, DisplayDriver, DisplayInfo, DmaCapable};
pub use event::{Event, EventCode, EventHandler};
pub use font::Font;
pub use input::{InputDriver, InputState, PointerData};
pub use layout::{Align, Direction, Layout, LayoutChild, Rect};
pub use state::State;
pub use style::Style;
