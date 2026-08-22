---
name: desktop-release-pipeline
description: >-
  Runbook for version management, cross-platform compilation (Windows, macOS, Linux), GitHub Actions CI/CD releases, and code signing in VulnRadar.
  Use this skill whenever preparing a new software release, bumping version tags, configuring GitHub Actions release workflows, or troubleshooting binary packaging.
---

# 📦 VulnRadar Desktop Release & Packaging Runbook

This skill provides step-by-step procedures for synchronizing versions, configuring automated CI/CD release builds, and distributing cross-platform binaries for VulnRadar.

---

## 🔢 1. Version Synchronization (The 3-File Rule)

Whenever preparing a new release (e.g. `v0.2.0`), the version number **must be identical** across all three configuration files:

1. **`package.json`**:
   ```json
   { "name": "vuln-radar", "version": "0.2.0" }
   ```
2. **`src-tauri/Cargo.toml`**:
   ```toml
   [package]
   name = "vuln-radar"
   version = "0.2.0"
   ```
3. **`src-tauri/tauri.conf.json`**:
   ```json
   { "version": "0.2.0" }
   ```

---

## 🚀 2. GitHub Actions Multi-Platform Release Workflow

The automated build pipeline compiles standalone binaries concurrently across 3 operating systems upon pushing a version tag (e.g., `git tag v0.2.0 && git push origin v0.2.0`).

### Target Matrix & Output Artifacts:
- **Windows (x64)**: `VulnRadar_0.2.0_x64-setup.exe` (NSIS) and `VulnRadar_0.2.0_x64_en-US.msi`
- **macOS (Universal / Apple Silicon + Intel)**: `VulnRadar_0.2.0_universal.dmg`
- **Linux (x64)**: `vuln-radar_0.2.0_amd64.AppImage` and `vuln-radar_0.2.0_amd64.deb`

---

## 🛡️ 3. Windows SmartScreen & Code Signing

### The SmartScreen Unknown Publisher Behavior
Because open-source projects without a commercial EV certificate cannot sign binaries with Microsoft-trusted hardware tokens, Windows displays an *"Unknown Publisher"* warning.

### Documentation & Mitigations:
1. **Document instructions clearly in `README.md`**:
   - Step 1: Click **More info**
   - Step 2: Click **Run anyway**
2. **Provide SHA256 Checksums**: Always generate `checksums.txt` in release assets so security analysts can verify binary integrity.

---

## 🔨 4. Local Build Testing

Before tagging a release, run local clean builds:

```bash
# Frontend type check & bundle build
npm run check
npm run build

# Standalone Tauri desktop binary compilation
npm run tauri build
```

Binary outputs will be located in:
- Windows: `src-tauri/target/release/bundle/nsis/` & `msi/`
- macOS: `src-tauri/target/release/bundle/dmg/`
- Linux: `src-tauri/target/release/bundle/appimage/` & `deb/`
