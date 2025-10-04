# Release Process

This document describes how to create and publish releases for the Veyra programming language.

> **Note:** For technical details about the GitHub Actions workflows, see [.github/WORKFLOWS.md](.github/WORKFLOWS.md)

## Automated Releases

### Creating a Release

1. **Tag the release**: Create and push a git tag following semantic versioning:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **Automatic build**: The GitHub Actions workflow will automatically:
   - Build binaries for all supported platforms
   - Create release archives with standard library and examples
   - Build the VS Code extension
   - Create a GitHub release with all artifacts

### Supported Platforms

The release workflow builds for the following platforms:

- **Linux**:
  - x86_64 (Intel/AMD 64-bit)
  - aarch64 (ARM 64-bit)
- **Windows**:
  - x86_64 (Intel/AMD 64-bit)  
  - aarch64 (ARM 64-bit)
- **macOS**:
  - x86_64 (Intel)
  - aarch64 (Apple Silicon)

## Manual Testing

### Test Release Build

You can test the release build process without creating a tag:

1. Go to the GitHub repository
2. Click "Actions" tab
3. Select "Test Release Build" workflow
4. Click "Run workflow"
5. Enter a test version number (e.g., "0.1.0-test")
6. Download and test the artifacts

### Local Testing

Use the provided scripts to build releases locally:

**Unix/Linux/macOS:**
```bash
./scripts/build-release.sh [version]
```

**Windows:**
```powershell
.\scripts\build-release.ps1 -Version [version]
```

## Release Contents

Each release archive contains:

- **Binaries** (in `bin/` directory):
  - `veyc` - Veyra compiler
  - `veyra-repl` - Interactive REPL
  - `veyra-fmt` - Code formatter
  - `veyra-lint` - Linter and static analyzer
  - `veyra-lsp` - Language Server Protocol implementation
  - `veyra-dbg` - Debugger
  - `veyra-pkg` - Package manager

- **Standard Library** (in `stdlib/` directory):
  - Core language functions
  - Collections, I/O, networking, and more

- **Examples** (in `examples/` directory):
  - Sample Veyra programs
  - Learning materials

- **Documentation**:
  - README.md
  - LICENSE
  - QUICK_START.md

## VS Code Extension

The VS Code extension is built separately and included as a `.vsix` file in releases. Users can install it by:

1. Downloading the `.vsix` file
2. Opening VS Code
3. Using "Extensions: Install from VSIX" command

## Troubleshooting

### Build Failures

- Check that all dependencies are properly specified in Cargo.toml files
- Ensure cross-compilation tools are available for ARM64 builds
- Verify that all binary names match between build scripts and Cargo.toml

### Missing Artifacts

- Check that the GitHub Actions workflow completed successfully
- Verify that all build steps produced the expected binaries
- Ensure upload steps didn't fail due to file path issues

### Version Mismatches

- Make sure the tag format matches `v*.*.*` (e.g., `v1.0.0`)
- Verify that version numbers in package.json and Cargo.toml are consistent