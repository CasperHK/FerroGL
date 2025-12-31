#![no_std]

//! FerroGL: High-Performance, Safe Embedded UI in Rust
//!
//! - 100% safe Rust, no C dependencies
//! - Zero-cost abstractions, no dynamic allocation
//! - Modern, Flexbox-inspired layout engine
//! - Hardware-agnostic driver interface
//! - Designed for embedded and bare-metal targets

pub mod layout;
pub mod widget;
pub mod driver;
pub mod state;
