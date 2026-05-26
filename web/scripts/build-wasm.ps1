# Build the atom-core crate to WebAssembly for the web demo.
#
# Outputs JS shim + .wasm to web/wasm/. Invoked locally on Windows; the
# Vercel Linux build uses the .sh sibling.
#
# Run from anywhere (resolves paths relative to the script).
$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$webDir    = Split-Path -Parent $scriptDir
$repoRoot  = Split-Path -Parent $webDir

# npm-spawned shells on Windows don't always inherit ~/.cargo/bin on PATH.
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if ((Test-Path $cargoBin) -and ($env:PATH -notlike "*$cargoBin*")) {
    $env:PATH = "$cargoBin;$env:PATH"
}

Push-Location $repoRoot
try {
    # --out-dir is resolved relative to the crate's manifest dir, so we
    # walk back up to the repo root and into web/wasm.
    wasm-pack build `
        --target web `
        --out-dir ../../web/wasm `
        crates/atom-core `
        -- --features wasm
    if ($LASTEXITCODE -ne 0) { throw "wasm-pack exited with $LASTEXITCODE" }
}
finally {
    Pop-Location
}
