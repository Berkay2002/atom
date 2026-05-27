# Watch crates/atom-core and rebuild the WASM bundle on every change.
#
# Run alongside `next dev` in a side terminal so atom-core edits propagate
# to the browser without manually re-running `npm run wasm`.
#
# Requires cargo-watch: `cargo install cargo-watch`.
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$webDir    = Split-Path -Parent $scriptDir
$repoRoot  = Split-Path -Parent $webDir

# npm-spawned shells on Windows don't always inherit ~/.cargo/bin on PATH.
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if ((Test-Path $cargoBin) -and ($env:PATH -notlike "*$cargoBin*")) {
    $env:PATH = "$cargoBin;$env:PATH"
}

if (-not (Get-Command cargo-watch -ErrorAction SilentlyContinue)) {
    Write-Host ""
    Write-Host "cargo-watch not found on PATH." -ForegroundColor Yellow
    Write-Host "Install it with:" -ForegroundColor Yellow
    Write-Host "    cargo install cargo-watch" -ForegroundColor Cyan
    Write-Host ""
    exit 1
}

Push-Location $repoRoot
try {
    # Watch the crate source from the repo root and re-invoke the existing
    # PowerShell build script on every change. `cargo watch` debounces
    # rapid edits by default (~500ms).
    cargo watch `
        --watch crates/atom-core `
        --shell "pwsh -NoProfile -File web/scripts/build-wasm.ps1"
    if ($LASTEXITCODE -ne 0) { throw "cargo watch exited with $LASTEXITCODE" }
}
finally {
    Pop-Location
}
