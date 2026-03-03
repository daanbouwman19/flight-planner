$ErrorActionPreference = "Stop"

# Configuration
$CoverageThreshold = 50
$OutputDir = "cov"

Write-Host "Running coverage analysis with threshold ${CoverageThreshold}%..."

# Ensure output directory exists
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
}

# Ensure cargo-llvm-cov is installed
if (-not (Get-Command "cargo-llvm-cov" -ErrorAction SilentlyContinue)) {
    Write-Host "Installing cargo-llvm-cov (this may take a minute)..."
    cargo install cargo-llvm-cov
}

# Run llvm-cov
# --workspace: Test all packages in the workspace
# --all-targets: Test all targets
# --lcov: Output Lcov format
# --output-path: Output file
# --fail-under-lines: Fail if coverage is below threshold
cargo llvm-cov --all-targets --workspace --lcov --output-path "$OutputDir/lcov.info" --fail-under-lines $CoverageThreshold

if ($LASTEXITCODE -eq 0) {
    Write-Host "Coverage check passed!" -ForegroundColor Green
} else {
    Write-Host "Coverage check failed." -ForegroundColor Red
    exit $LASTEXITCODE
}
