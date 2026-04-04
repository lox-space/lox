<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# @lox-space/wasm

WebAssembly bindings for Lox Space orbital mechanics, providing high-performance calculations for web applications.

## Overview

This package contains Rust code compiled to WebAssembly that can be used by the Vite-based apps in the Kerolox monorepo (particularly the planetarium app). It provides performance-critical orbital mechanics calculations.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) - Install with: `cargo install wasm-pack`

## Building

Build the WASM package:

```bash
pnpm build
```

This runs `wasm-pack build --target web --out-dir pkg` which:
- Compiles Rust to WebAssembly
- Generates JavaScript bindings
- Creates TypeScript type definitions
- Outputs everything to the `pkg/` directory

## Testing

Run Rust tests:

```bash
pnpm test
```

Or directly with cargo:

```bash
cargo test
```

## Usage in Vite Apps

The planetarium app has been configured with `vite-plugin-wasm` to support WASM imports.

### Basic Import

```typescript
import init, { greet, orbital_velocity } from "@lox-space/wasm";

// Initialize the WASM module (required before use)
await init();

// Use exported functions
const message = greet("World");
console.log(message); // "Hello, World! Welcome to Lox Space WASM."

// Calculate orbital velocity (μ = 398600.4418 km³/s², r = 6700 km)
const velocity = orbital_velocity(398600.4418, 6700.0);
console.log(`Orbital velocity: ${velocity.toFixed(2)} km/s`);
```

### In React Components

```typescript
import { useEffect, useState } from "react";
import init, { orbital_velocity } from "@lox-space/wasm";

function OrbitalCalculator() {
  const [wasmReady, setWasmReady] = useState(false);

  useEffect(() => {
    init().then(() => setWasmReady(true));
  }, []);

  if (!wasmReady) {
    return <div>Loading WASM...</div>;
  }

  const velocity = orbital_velocity(398600.4418, 6700.0);

  return <div>Orbital velocity: {velocity.toFixed(2)} km/s</div>;
}
```

## Available Functions

### `greet(name: string): string`

A simple greeting function demonstrating string handling.

### `orbital_velocity(gravitational_parameter: f64, radius: f64): f64`

Calculate orbital velocity using the formula: `v = sqrt(μ / r)`

- `gravitational_parameter`: Standard gravitational parameter (μ) in km³/s²
- `radius`: Orbital radius in km
- Returns: Orbital velocity in km/s

### `deg_to_rad(degrees: f64): f64`

Convert degrees to radians.

### `rad_to_deg(radians: f64): f64`

Convert radians to degrees.

## Architecture

- **Source**: Rust code in `src/lib.rs`
- **Build Output**: JavaScript bindings and WASM binary in `pkg/`
- **Target**: `web` - optimized for browser use with ES modules
- **Build Tool**: wasm-pack with wasm-bindgen

## Development Notes

- The WASM module automatically initializes panic hooks for better error messages in the browser console
- Functions are decorated with `#[wasm_bindgen]` to expose them to JavaScript
- The build output includes TypeScript definitions for type safety
- The Vite plugin handles async WASM initialization automatically

## Adding New Functions

1. Add your function to `src/lib.rs` with the `#[wasm_bindgen]` attribute:

```rust
#[wasm_bindgen]
pub fn your_function(param: f64) -> f64 {
    // Your implementation
    param * 2.0
}
```

2. Rebuild the package:

```bash
pnpm build
```

3. The function will be automatically exported and available for import in TypeScript/JavaScript.

## License

MIT
