<div align="center">

# 🐰 Chunkkin

**High-performance world pre-generator plugin for [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin), written in Rust and compiled to WebAssembly.**

[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![WASM](https://img.shields.io/badge/Target-wasm32--wasip2-654ff0?logo=webassembly&logoColor=white)](https://doc.rust-lang.org/rustc/platform-support/wasm32-was.html)
[![Pumpkin](https://img.shields.io/badge/Server-Pumpkin-ff8c00)](https://github.com/Pumpkin-MC/Pumpkin)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue)](LICENSE-MIT)
[![Languages](https://img.shields.io/badge/Languages-EN%20%7C%20RU-green)](README-RU.md)

*Pre-generate your worlds before players arrive - spiral-first, batched, resumable.*

[Features](#-features) · [Commands](#-commands) · [Configuration](#%EF%B8%8F-configuration) · [Build](#%EF%B8%8F-build--install) · [Русская версия](README-RU.md)

</div>

---

## 💡 Why

Minecraft servers lag the most when a player walks into ungenerated terrain: the server has to generate chunks *and* serve them in the same tick. Chunkkin removes that spike entirely - it walks your world's chunks **outward from the center in a spiral** (so spawn is ready first) in **tunable batches** (so TPS stays flat), **saves progress to disk** (so a restart never loses work), and reports everything live.

Think Chunky, but native to the Pumpkin server core: Rust, WebAssembly, zero JVM.

## ✨ Features

- 🌀 **Spiral-first traversal** - generation starts at the center and expands in concentric rings; spawn is playable before the far corners exist.
- 🟦 **Square & rectangle regions** - one radius for a square, two radii (X/Z) for a rectangle, just like Chunky's selection model.
- ⚡ **Batched generation** - a configurable number of chunks per server tick keeps TPS smooth while saturating the generation pipeline.
- 💾 **Persistent, resumable tasks** - progress is checkpointed to disk; a crash or restart resumes exactly where it left off.
- 🌍 **Localization** - English by default, Russian via one config field; every message, label and help line is translated.
- 📊 **Live telemetry** - progress bar, speed (chunks/s), ETA and elapsed time in chat and console.
- ♻️ **Hot config reload** - `/chunkkin reload` applies language and engine settings without a restart.
- 🔒 **Sandboxed** - runs in Wasmtime with filesystem access limited to the plugin's own data folder.

## 📋 Commands

Root command: `/chunkkin` (alias `/ck`). Required permission: `chunkkin:command` (default: allow).

### Setting up the selection

The selection persists in `config.json` and survives server restarts - set it once, run it anytime.

| Command | Description | Example |
|---|---|---|
| `/ck world [world]` | Target world (your current world when omitted) | `/ck world world_nether` |
| `/ck center [x z]` | Generation center in blocks (your position when omitted; `~` supported) | `/ck center 1000 -2000` |
| `/ck radius <r> [rz]` | Radius in blocks; two values create a rectangle | `/ck radius 5000` / `/ck radius 5000 2000` |
| `/ck selection` | Show the current selection | `/ck selection` |

> [!TIP]
> Defaults: world `world`, center `0 0`, radius `500` (a 1000 × 1000 block square).

### Running the task

| Command | Description | Example |
|---|---|---|
| `/ck start` | Start generating with the current selection | `/ck start` |
| `/ck pause` | Pause; progress is saved | `/ck pause` |
| `/ck continue` | Resume the paused task | `/ck continue` |
| `/ck cancel` | Cancel and reset the task | `/ck cancel` |
| `/ck status` | Progress report; a repeat call toggles chat progress on/off | `/ck status` |
| `/ck reload` | Reload `config.json` from disk without a restart | `/ck reload` |
| `/ck help` | Command reference in chat | `/ck help` |

## ⚙️ Configuration

Created automatically at `plugins/data/chunkkin/config.json`:

```json
{
  "language": "en",
  "selection": {
    "world": "world",
    "center_x": 0,
    "center_z": 0,
    "radius_x": 500,
    "radius_z": 500
  },
  "batch_size": 16,
  "log_interval_ticks": 100,
  "auto_save_interval_chunks": 1000,
  "auto_resume_on_startup": true,
  "console_log": true
}
```

| Option | Default | Description |
|---|---|---|
| `language` | `"en"` | Message language: `"en"` or `"ru"`. Missing fields in old configs get defaults. |
| `selection` | see above | Current selection; changed by the `/ck world` / `/ck center` / `/ck radius` commands. |
| `batch_size` | `16` | Chunks generated per server tick (20 ticks = 1 s). Raise for faster generation, lower if TPS drops. |
| `log_interval_ticks` | `100` | Ticks between progress messages (100 ticks = 5 s). |
| `auto_save_interval_chunks` | `1000` | Every N generated chunks, the world is flushed to disk. |
| `auto_resume_on_startup` | `true` | Automatically resume an interrupted task when the server starts. |
| `console_log` | `true` | Also print one-line progress to the server console (chat progress is toggled by `/ck status`). |

## 🛠️ Build & Install

### Requirements

- Rust 1.85+ (Edition 2024)
- The WebAssembly WASI target:

```bash
rustup target add wasm32-wasip2
```

### Building

```bash
cargo build --target wasm32-wasip2 --release
```

The artifact lands at `target/wasm32-wasip2/release/chunkkin.wasm`.

> [!NOTE]
> `pumpkin-plugin-api` is pinned to a specific Pumpkin commit in `Cargo.toml`. If you run a different server build, update the `rev` (or switch to `branch = "master"`) so the WIT interface matches your server.

### Installing

1. Copy `chunkkin.wasm` into the server's `plugins/` directory.
2. Make sure plugins are enabled in `pumpkin.toml`:
   ```toml
   [plugins]
   enabled = true
   ```
3. Start the server (or hot-reload plugins) and approve the plugin's permissions on first launch.

## 🧭 How it works

```
        ┌─────────────┐     ┌──────────────────┐     ┌──────────────────┐
 /ck ──▶│  Selection  │────▶│  SpiralIterator  │────▶│  TaskManager     │
 start  │ (config.json│     │  (O(1) index →   │     │  batch per tick, │
        │  persists)  │     │   ring offsets)  │     │  save, resume)   │
        └─────────────┘     └──────────────────┘     └────────┬─────────┘
                                                         every tick
                                                  ┌───────────────▼────────┐
                                                  │  Pregenerator          │
                                                  │  heightmap query →     │
                                                  │  chunk load/generation │
                                                  └────────────────────────┘
```

1. **Selection** - world, center and radii are stored in `config.json` (Chunky-style).
2. **SpiralIterator** - converts a running index into ring offsets with O(1) arithmetic; chunks are visited center-out.
3. **TaskManager** - processes `batch_size` coordinates per tick, checkpoints progress, auto-saves the world, and feeds telemetry.
4. **Pregenerator** - touches each chunk's heightmap, which makes Pumpkin's chunk system load or generate it.

## 📄 License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.

---

<div align="center">
<sub>Built with 🧡 for the <a href="https://github.com/Pumpkin-MC/Pumpkin">Pumpkin</a> server core · <a href="README-RU.md">Читать на русском</a></sub>
</div>
