# Blitz Diff

A mod manager for **World of Tanks Blitz**, written in Rust with [egui](https://github.com/emilk/egui). Deploy local `.zip` mods to your Steam or Wargaming Game Center (WGC) `Data/` folder, manage a mod library with load order, revert safely, and verify file integrity.

---

## Features

- **Dual client support** — Deploy to Steam, WGC, or both; leave a path blank to disable that client
- **Mod library** — Scan a local folder for `.zip` mods, enable/disable, reorder load order
- **Metadata** — Read `blitz-mod.json` from inside the zip or a sidecar file next to it
- **Apply All Enabled** — Deploy enabled mods in load order with conflict warnings
- **Revert** — Restore from local backups, opposite client, or remove mod-added files
- **Integrity scan** — SHA-256 comparison of tracked files vs live game and optional zip check
- **Profiles** — Save and apply named mod presets
- **Auto-detect paths** — Find Steam/WGC installs via registry and library folders
- **Backups browser** — Restore individual files from `blitz_diff_local_backups/`
- **Drag-and-drop** — Drop `.zip` files onto the window to add to library

---

## DAVA / Data Path Rules

WoT Blitz uses the **DAVA Framework**. Assets are typically `.dvpl` blobs (e.g. `texture.pvr.dvpl`, `list.xml.dvpl`). Blitz Diff copies them **byte-for-byte** — no decompression or editing.

When extracting mods, paths are resolved relative to the **`Data/`** anchor inside each zip entry:

| Zip entry | Deployed to |
|-----------|-------------|
| `MyMod/Data/Gfx/foo.pvr.dvpl` | `Data/Gfx/foo.pvr.dvpl` |
| `Data/Data/XML/list.xml.dvpl` | `Data/XML/list.xml.dvpl` |
| `Gfx/foo.pvr.dvpl` (no `Data/`) | Skipped with warning |

---

## Mod Metadata (`blitz-mod.json`)

Optional manifest inside the zip or as `{zip-name}.blitz-mod.json` beside it:

```json
{
  "name": "HD Minimap",
  "version": "2.1.0",
  "author": "ModAuthor",
  "description": "Replaces minimap textures",
  "game_version": "11.5.0",
  "thumbnail": "thumb.png"
}
```

If no manifest exists, the zip filename is used as the mod name.

---

## Build & Run

Requires [Rust](https://rustup.rs/) (edition 2024).

```bash
cargo build --release
cargo run
```

Release builds hide the console on Windows.

---

## Directory Layout

### Local workspace (next to executable)

```text
blitz-diff.exe
blitz_diff_local_backups/
  steam/          # Original files cached before Steam overwrites
  wgc/            # Original files cached before WGC overwrites
mods/             # Default mod library folder
```

### AppData (`%APPDATA%\blitz-diff\`)

```text
config.toml       # Game paths, library folder, active profile
catalog.toml      # Mod library (enabled, load order, metadata)
state_steam.toml  # Per-file hashes and ownership (Steam)
state_wgc.toml    # Per-file hashes and ownership (WGC)
profiles.toml     # Saved mod presets
```

---

## Usage

1. Open **Settings** → configure Steam/WGC `Data/` paths (or use **Auto-detect**)
2. Set a **Mods Library Folder** and add `.zip` files via **Library** tab or drag-and-drop
3. Enable mods and set load order (▲/▼)
4. Use **Apply All Enabled** on the Operations tab, or apply a single zip manually
5. **Revert** restores tracked files; **Integrity Scan** checks for drift after game patches

Close World of Tanks Blitz before deploying — the app warns if the game process is running.