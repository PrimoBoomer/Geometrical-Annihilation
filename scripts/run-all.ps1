#requires -Version 7
<#
Builds and launches the full Geometrical Annihilation dev stack:
  1. Cargo build the Rust server (release)
  2. Export the Godot client to HTML5
  3. Start the Rust server (ws://127.0.0.1:8080)
  4. Start the Python static web server (http://127.0.0.1:8081)
  5. Open the default browser on the game URL

Press Ctrl+C in this window to terminate both background processes.
Override the Godot binary path via $env:GODOT_BIN if it's not in PATH.
#>

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$ServerDir = Join-Path $RepoRoot "server"
$ClientDir = Join-Path $RepoRoot "client"
$ExportDir = Join-Path $ClientDir "export/web"
$ServerExe = Join-Path $ServerDir "target/release/server.exe"
$ServeWeb  = Join-Path $RepoRoot "scripts/serve-web.py"
$GameUrl   = "http://127.0.0.1:8081/"

function Resolve-GodotBin {
    if ($env:GODOT_BIN -and (Test-Path $env:GODOT_BIN)) {
        return $env:GODOT_BIN
    }
    foreach ($name in @("godot", "godot.exe", "Godot_v4.6-stable_win64.exe", "Godot_v4.6.2-stable_win64.exe")) {
        $cmd = Get-Command $name -ErrorAction SilentlyContinue
        if ($cmd) { return $cmd.Source }
    }
    throw "Godot executable not found. Set GODOT_BIN env var or add godot to PATH."
}

function Resolve-PythonBin {
    if ($env:PYTHON_BIN -and (Test-Path $env:PYTHON_BIN)) {
        return $env:PYTHON_BIN
    }
    $candidates = @()
    foreach ($name in @("python", "python3", "py")) {
        $cmd = Get-Command $name -ErrorAction SilentlyContinue
        if ($cmd) { $candidates += $cmd.Source }
    }
    $candidates += @(
        (Join-Path $env:LOCALAPPDATA "Python/bin/python.exe"),
        (Join-Path $env:LOCALAPPDATA "Programs/Python/Python312/python.exe"),
        (Join-Path $env:LOCALAPPDATA "Programs/Python/Python313/python.exe"),
        (Join-Path $env:LOCALAPPDATA "Programs/Python/Python314/python.exe")
    )
    foreach ($p in $candidates) {
        if (-not $p -or -not (Test-Path $p)) { continue }
        if ($p -like "*WindowsApps*python*.exe") { continue }  # Microsoft Store placeholder
        try {
            $null = & $p --version 2>$null
            if ($LASTEXITCODE -eq 0) { return $p }
        } catch {}
    }
    throw "No working Python interpreter found. Set PYTHON_BIN or install Python."
}

Write-Host "==> Building Rust server (release)" -ForegroundColor Cyan
cargo build --release --manifest-path (Join-Path $ServerDir "Cargo.toml")
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

Write-Host "==> Exporting Godot client to $ExportDir" -ForegroundColor Cyan
$null = New-Item -ItemType Directory -Force -Path $ExportDir
$Godot = Resolve-GodotBin
Write-Host "    using Godot: $Godot"

$TemplatesDir = Join-Path $env:APPDATA "Godot/export_templates/4.6.2.stable"
if (-not (Test-Path (Join-Path $TemplatesDir "web_release.zip"))) {
    Write-Host "    Web export templates missing; running install-export-templates.ps1" -ForegroundColor Yellow
    & (Join-Path $PSScriptRoot "install-export-templates.ps1")
}

& $Godot --headless --path $ClientDir --export-release "Web" (Join-Path $ExportDir "index.html")
if ($LASTEXITCODE -ne 0) { throw "Godot export failed (templates installed? preset name 'Web' present?)" }

Write-Host "==> Starting Rust server in a new window" -ForegroundColor Cyan
$serverArgs = @(
    "-NoExit",
    "-Command",
    "`$env:RUST_LOG='info'; & '$ServerExe'"
)
$serverProc = Start-Process -FilePath "pwsh" -ArgumentList $serverArgs -PassThru

Write-Host "==> Starting Python static web server in a new window" -ForegroundColor Cyan
$Python = Resolve-PythonBin
Write-Host "    using Python: $Python"
$webArgs = @(
    "-NoExit",
    "-Command",
    "Set-Location '$RepoRoot'; & '$Python' '$ServeWeb'"
)
$webProc = Start-Process -FilePath "pwsh" -ArgumentList $webArgs -PassThru

Start-Sleep -Milliseconds 500
Write-Host "==> Opening browser at $GameUrl" -ForegroundColor Cyan
Start-Process $GameUrl

Write-Host ""
Write-Host "Server PID: $($serverProc.Id)   Web PID: $($webProc.Id)" -ForegroundColor Green
Write-Host "Press Ctrl+C in this window to stop everything." -ForegroundColor Green

try {
    while ($true) {
        if ($serverProc.HasExited -and $webProc.HasExited) { break }
        Start-Sleep -Seconds 1
    }
} finally {
    Write-Host "==> Stopping background processes" -ForegroundColor Yellow
    foreach ($p in @($serverProc, $webProc)) {
        if ($p -and -not $p.HasExited) {
            try { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue } catch {}
        }
    }
}
