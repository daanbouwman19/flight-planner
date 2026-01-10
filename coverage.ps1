$ErrorActionPreference = "Stop"

# Configuration
$CoverageThreshold = 80
$OutputDir = "cov"

Write-Host "Running coverage analysis with threshold ${CoverageThreshold}%..."

# Ensure output directory exists
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
}

# Run tarpaulin
# --skip-clean: Don't clean build artifacts (faster re-runs)
# --all-targets: Test all targets
# --jobs 1: Run sequentially to avoid potential race conditions in tests
# --out Lcov: Output Lcov format
# --output-dir: Output directory
# --fail-under: Fail if coverage is below threshold
cargo tarpaulin --skip-clean --all-targets --out Lcov --output-dir $OutputDir --fail-under $CoverageThreshold --exclude-files src/gui/*

if ($LASTEXITCODE -eq 0) {
    Write-Host "Coverage check passed!" -ForegroundColor Green
} else {
    Write-Host "Coverage check failed." -ForegroundColor Red
    exit $LASTEXITCODE
}
