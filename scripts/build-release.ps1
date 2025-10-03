# Veyra Release Script for Windows
# This script helps create a local release build for testing

param(
    [string]$Version = "0.1.0-local"
)

$Platform = "windows"
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
$ArtifactName = "veyra-$Platform-$Arch"
$ReleaseDir = "release"

Write-Host "Building Veyra v$Version for $Platform-$Arch"

# Clean previous builds
if (Test-Path $ReleaseDir) {
    Remove-Item -Recurse -Force $ReleaseDir
}
New-Item -ItemType Directory -Force -Path "$ReleaseDir\bin" | Out-Null
New-Item -ItemType Directory -Force -Path "$ReleaseDir\stdlib" | Out-Null
New-Item -ItemType Directory -Force -Path "$ReleaseDir\examples" | Out-Null

# Build compiler
Write-Host "Building compiler..."
Set-Location compiler
cargo build --release
Set-Location ..

# Build tools
Write-Host "Building tools..."
Set-Location tools
cargo build --release
Set-Location ..

# Copy binaries
Write-Host "Copying binaries..."
Copy-Item "compiler\target\release\veyc.exe" "$ReleaseDir\bin\"
Copy-Item "tools\target\release\veyra-repl.exe" "$ReleaseDir\bin\"
Copy-Item "tools\target\release\veyra-dbg.exe" "$ReleaseDir\bin\"
Copy-Item "tools\target\release\veyra-lint.exe" "$ReleaseDir\bin\"
Copy-Item "tools\target\release\veyra-fmt.exe" "$ReleaseDir\bin\"
Copy-Item "tools\target\release\veyra-lsp.exe" "$ReleaseDir\bin\"
Copy-Item "tools\target\release\veyra-pkg.exe" "$ReleaseDir\bin\"

# Copy standard library and examples
Write-Host "Copying standard library and examples..."
Copy-Item -Recurse "stdlib\*" "$ReleaseDir\stdlib\"
Copy-Item -Recurse "examples\*" "$ReleaseDir\examples\"

# Copy documentation
Write-Host "Copying documentation..."
Copy-Item "README.md" "$ReleaseDir\"
Copy-Item "LICENSE" "$ReleaseDir\"
Copy-Item "QUICK_START.md" "$ReleaseDir\"

# Create archive
Write-Host "Creating archive..."
Set-Location $ReleaseDir
Compress-Archive -Path * -DestinationPath "..\$ArtifactName.zip"
Set-Location ..

Write-Host "Release build completed: $ArtifactName.zip"
Write-Host ""
Write-Host "Contents:"
Write-Host "- Compiler: veyc.exe"
Write-Host "- REPL: veyra-repl.exe"
Write-Host "- Debugger: veyra-dbg.exe"
Write-Host "- Linter: veyra-lint.exe"
Write-Host "- Formatter: veyra-fmt.exe"
Write-Host "- Language Server: veyra-lsp.exe"
Write-Host "- Package Manager: veyra-pkg.exe"
Write-Host "- Standard Library"
Write-Host "- Examples"
Write-Host "- Documentation"
Write-Host ""
Write-Host "To install:"
Write-Host "1. Extract the archive"
Write-Host "2. Add the bin directory to your PATH"