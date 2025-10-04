# GitHub Actions Workflows Documentation

This document describes the GitHub Actions workflows used in this repository and important notes about their implementation.

## Workflows Overview

### 1. Release Workflow (`release.yml`)

**Trigger:** Push of tags matching `v*.*.*` pattern or manual dispatch

**Purpose:** Builds and publishes official releases for all supported platforms

**Jobs:**
- `build-release`: Builds binaries for all platforms (Linux x64/ARM64, Windows x64/ARM64, macOS x64/ARM64)
- `build-vscode-extension`: Builds the VS Code extension
- `create-release`: Creates GitHub release with all artifacts

**Important:** This workflow uses `softprops/action-gh-release@v2` for creating releases. Do NOT use `actions/github-script@v7` for uploading release assets as it requires manual API calls and is error-prone.

### 2. Test Release Workflow (`test-release.yml`)

**Trigger:** Manual workflow dispatch only

**Purpose:** Test the release build process without creating an actual release

**Jobs:**
- `test-build`: Builds binaries for main platforms (Linux x64, Windows x64, macOS x64)
- `test-vscode-extension`: Builds the VS Code extension

**Note:** This workflow only uploads artifacts for testing, it does not create a GitHub release.

## Common Pitfalls and Best Practices

### ❌ DO NOT Use github-script for Release Asset Upload

**Bad Example (causes syntax errors):**
```yaml
- name: Upload VS Code Extension
  uses: actions/github-script@v7
  with:
    script: |
      const fs = require('fs');
      const path = require('path');
      
      const artifactPath = path.join(process.cwd(), 'tools', 'vscode_extension', 'veyra-lang-0.1.5.vsix');
      const artifactName = 'veyra-lang-0.1.5.vsix';
      
      const fileData = fs.readFileSync(artifactPath);
      
      const uploadAsset = await github.rest.repos.uploadReleaseAsset({
        owner: context.repo.owner,
        repo: context.repo.repo,
        release_id: ,  // ❌ This causes "Unexpected token ','" error
        name: artifactName,
        data: fileData
      });
```

**Problems with this approach:**
1. Requires manual release_id lookup
2. Prone to syntax errors
3. Requires complex file handling
4. More code to maintain

### ✅ DO Use softprops/action-gh-release

**Good Example (recommended):**
```yaml
- name: Create Release with Assets
  uses: softprops/action-gh-release@v2
  with:
    files: |
      artifacts/veyra-linux-x64/*
      artifacts/veyra-linux-arm64/*
      artifacts/veyra-windows-x64/*
      artifacts/veyra-windows-arm64/*
      artifacts/veyra-macos-x64/*
      artifacts/veyra-macos-arm64/*
      artifacts/vscode-extension/*
    body: |
      ## Release Notes
      ...
    draft: false
    prerelease: false
```

**Benefits:**
1. Automatic release creation
2. Simple glob pattern file matching
3. Handles all upload complexity internally
4. Well-tested and maintained

## Artifact Handling

### Upload Artifacts
Use `actions/upload-artifact@v4` in build jobs:
```yaml
- name: Upload artifact
  uses: actions/upload-artifact@v4
  with:
    name: artifact-name
    path: path/to/artifact
    retention-days: 1
```

### Download Artifacts
Use `actions/download-artifact@v4` in release job:
```yaml
- name: Download all artifacts
  uses: actions/download-artifact@v4
  with:
    path: artifacts
```

## Version Handling

The workflows extract version from git tags:
```yaml
- name: Get version from tag
  id: get_version
  run: |
    if [ "${{ github.ref_type }}" = "tag" ]; then
      echo "version=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT
    else
      echo "version=0.1.0-dev" >> $GITHUB_OUTPUT
    fi
```

Use this version in subsequent steps: `${{ steps.get_version.outputs.version }}`

## Troubleshooting

### Problem: "Unexpected token" syntax errors in workflow runs

**Cause:** Using `actions/github-script@v7` with incomplete or incorrect JavaScript

**Solution:** Replace with `softprops/action-gh-release@v2` as shown above

### Problem: Release created but assets not uploaded

**Cause:** Incorrect file paths or glob patterns in `files:` configuration

**Solution:** 
1. Add a debug step to list artifact structure: `run: ls -R artifacts`
2. Verify the paths match the actual artifact structure
3. Use glob patterns like `artifacts/artifact-name/*` to match all files in artifact

### Problem: VS Code extension build fails

**Cause:** Node.js version compatibility or missing dependencies

**Solution:**
1. Ensure Node.js 20+ is used (required for @vscode/vsce)
2. Run `npm install` before `vsce package`
3. Install vsce globally: `npm install -g @vscode/vsce`

## Historical Note

**Previous Issue (October 2025):**
Earlier versions of the release workflow used `actions/github-script@v7` with incomplete JavaScript code that caused syntax errors:
```
SyntaxError: Unexpected token ','
    at new AsyncFunction (<anonymous>)
```

This was caused by having `release_id: ,` (missing value) in the github-script code. The workflow has been updated to use `softprops/action-gh-release@v2` which eliminates this entire class of errors.

## References

- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)
- [actions/upload-artifact](https://github.com/actions/upload-artifact)
- [actions/download-artifact](https://github.com/actions/download-artifact)
- [GitHub Actions Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
