# FindApps

Universal Linux application manager. Detects, organizes, and uninstalls programs installed via **APT**, **DNF**, **Flatpak**, **Snap**, **AppImage**, and **manual** installs, with a native GTK4/libadwaita interface.

> MVP 0.1.0 — focused on discovery and management. Install and update flows are prepared in the architecture but not yet implemented.

## Supported distributions

| Family | Priority | Typical backends |
|--------|----------|------------------|
| Debian / Ubuntu and derivatives | High | APT, Snap, Flatpak, AppImage, Manual |
| Fedora / RHEL and derivatives | High | DNF, Flatpak, AppImage, Manual |
| Arch, openSUSE, Nix | Future | Extensible architecture ready |

The UI contains **no** distribution-specific logic. Backends are detected automatically; missing backends are omitted without errors.

## Implemented backends

| Backend | Detect | List | Uninstall | Install / Update |
|---------|--------|------|-----------|------------------|
| APT | Yes | Yes | Yes (pkexec) | Stub |
| DNF | Yes | Yes | Yes (pkexec) | Stub |
| Flatpak | Yes | Yes | Yes | Stub |
| Snap | Yes | Yes | Yes (pkexec) | Stub |
| AppImage | Yes | Yes | Yes (files under `$HOME`) | Stub |
| Manual | Yes | Yes | Limited (`~/.local`) | Stub |

## System dependencies

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install build-essential pkg-config \
  libgtk-4-dev libadwaita-1-dev libglib2.0-dev \
  policykit-1
```

### Fedora

```bash
sudo dnf install gcc pkg-config \
  gtk4-devel libadwaita-devel glib2-devel \
  polkit
```

You also need [Rust](https://rustup.rs/) (1.75+ recommended):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Development

```bash
# Clone / enter the project
cd findApps

# (recommended) system deps installed — see section above
# If *-dev headers cannot be installed via apt/dnf, use:
#   source scripts/dev-env.sh

# Run in development mode
cargo run

# Release build
cargo build --release

# Binary
./target/release/findapps
```

### Useful variables

```bash
RUST_LOG=debug cargo run   # detailed logs on stderr and in ~/.local/share/findapps/logs/
```

## Tests

```bash
cargo test
```

Tests use **mocks** for package managers and temporary fixtures (AppImage/desktop). They do not change the real system.

## Packaging

Skeletons live under `packaging/`:

### Snap (Ubuntu App Center)

Requires `snapcraft` and an account on [snapcraft.io](https://snapcraft.io) with the developer agreement signed.

> **Important:** `snapcraft` fails if the project path contains spaces. Use the script below, which packs from a clean directory.

```bash
# Register the name (once)
snapcraft login
snapcraft register findapps

# Pack (recommended — avoids the space-in-path bug)
./scripts/build-snap.sh

# Or manually from a path without spaces:
#   rsync -a --exclude target --exclude .git ./ ~/findapps-snap-build/
#   cd ~/findapps-snap-build && sudo snapcraft pack --destructive-mode

# Install locally for testing (use the real filename)
sudo snap install --dangerous --classic ./findapps_*.snap

# Publish (after classic review by Canonical)
snapcraft upload --release=edge ./findapps_*.snap
```

The snap uses `confinement: classic` because it must access host package managers (APT, Flatpak, Snap, pkexec).

### Flatpak

```bash
flatpak-builder --user --install build-dir packaging/flatpak/br.com.findapps.FindApps.yml
```

### .deb (skeleton)

Files in `packaging/deb/`. Integrate with `dh` / copy into `debian/` as required by your packaging flow.

### AppImage (skeleton)

1. `cargo build --release`
2. Use `packaging/appimage/AppImageBuilder.yml` with [appimage-builder](https://appimage-builder.readthedocs.io/) or linuxdeploy.

Desktop, metainfo, and PolicyKit files are under `data/`:

- `br.com.findapps.FindApps.desktop`
- `br.com.findapps.FindApps.metainfo.xml`
- `br.com.findapps.FindApps.policy`

## Security

- Commands use **separate arguments** (no shell).
- Package IDs are validated before operations.
- Elevation only for the operation (`pkexec` / PolicyKit), never `sudo` for the whole process.
- Explicit confirmation before uninstall, with a clear description of the backend and operation.

## Architecture

```text
                 FindApps
                     │
              PackageManager
                     │
       ┌─────────────┼─────────────┐
      APT          DNF          Flatpak
       │             │             │
      Snap        AppImage       Manual
```

Each backend implements the `PackageBackend` trait (`detect`, `list_installed`, `get_details`, `uninstall`, `install`, `update`, `check_updates`).

## Roadmap

- [ ] Install with method selection (Flatpak / Snap / APT / …)
- [ ] Detect and apply updates
- [ ] Arch / pacman backend
- [ ] openSUSE / zypper backend
- [ ] Nix backend
- [ ] Official RPM packages and AUR
- [x] i18n (English default + Chinese, Spanish, Hindi, Arabic, Portuguese, Russian)

## License

GPL-3.0-or-later
