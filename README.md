# FerroGL: The Future of High-Performance, Safe Embedded UI in Rust
In the world of embedded systems, LVGL has long been the gold standard. However, as the industry shifts towards memory safety and modernization, developers are increasingly looking for a solution that combines the efficiency of C with the uncompromising safety of Rust.
Introducing FerroGL — a next-generation, lightweight, and hardware-agnostic UI library built from the ground up in 100% safe Rust.

## 🚀 Why FerroGL?
While existing solutions often rely on complex C-bindings or resource-heavy web engines, FerroGL is engineered for the "Bare Metal" reality. It bridges the gap between high-level developer experience and low-level performance.

### 🛡️ Built-in Memory Safety
Leveraging Rust's ownership model, FerroGL eliminates common pitfalls such as buffer overflows, null pointer dereferencing, and memory leaks that plague traditional C-based UI libraries.

### ⚡ Zero-Cost Abstractions
FerroGL is designed with a no_std first philosophy. It provides a rich set of widgets—from buttons to complex charts—without requiring dynamic memory allocation (no_alloc), making it ideal for microcontrollers with extremely limited RAM.

### 🎨 Modern Layout Engine
Say goodbye to manual coordinate calculations. FerroGL introduces a simplified Flexbox-inspired layout engine optimized for embedded CPUs, allowing for responsive designs that adapt to any screen resolution seamlessly.

### 🔌 Hardware-Agnostic & DMA-Ready
Whether you are using an ESP32, an STM32, or a high-end RISC-V processor, FerroGL’s plug-and-play driver interface integrates effortlessly with embedded-graphics. It is built to leverage DMA (Direct Memory Access) for tear-free, buttery-smooth animations.

### 🛠️ Key Features at a Glance (2025 Roadmap)
Pure Rust Core: No C dependencies, making cross-compilation a breeze.
Sub-pixel Rendering: Crisp typography and smooth anti-aliased shapes.
Reactive State Management: An intuitive, declarative API inspired by modern web frameworks but optimized for binary size.
Extreme Portability: Runs on everything from monochrome OLEDs to full-color TFT displays.

### 🌍 A Tool for the Next Decade
As we move into 2025 and beyond, the complexity of IoT devices continues to grow. Developers deserve a UI toolchain that is as robust as the hardware it runs on. FerroGL isn't just another library; it’s a commitment to building a safer, faster, and more reliable embedded ecosystem.

---

### 📦 Get Started
FerroGL is currently in active development. We are looking for early adopters and contributors who are passionate about pushing the boundaries of what Rust can do on hardware.
Join the movement. Let’s oxidize the UI layer together.

---

## 🖥️ Supported Platforms

FerroGL is initially targeting the following CPU architectures and microcontroller platforms:
- **RISC-V**
- **ARM**
- **MCP**

Support for additional platforms will be added as the project evolves.

---

## 🧪 Running & Testing

FerroGL is a `no_std` Rust library, so it is primarily intended for embedded and bare-metal targets. However, you can build and test the core logic on your development machine:

```sh
# Build the library (native target)
cargo build

# Run tests (if any are implemented)
cargo test
```

For embedded targets, add FerroGL as a dependency in your project's `Cargo.toml` and follow your platform's build and flashing instructions.

Example for adding FerroGL to your project:

```toml
[dependencies]
ferrogl = { path = "../FerroGL" }
```

---