#requires -Version 7
<#
Downloads and installs the Godot 4.6.2-stable export templates into the
user's appdata so that `--export-release` works headlessly.

Idempotent: skips download if the .tpz cache already exists, and only
extracts the platform-specific zips into the target folder.
#>

param(
    [string]$Version = "4.6.2-stable",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$TplDir = Join-Path $env:APPDATA "Godot/export_templates/$($Version.Replace('-', '.'))"
$WebRelease = Join-Path $TplDir "web_release.zip"

if ((Test-Path $WebRelease) -and -not $Force) {
    Write-Host "Export templates already installed at $TplDir" -ForegroundColor Green
    exit 0
}

$Url = "https://github.com/godotengine/godot-builds/releases/download/$Version/Godot_v${Version}_export_templates.tpz"
$CacheDir = Join-Path $env:LOCALAPPDATA "GeometricalAnnihilation/cache"
$null = New-Item -ItemType Directory -Force -Path $CacheDir
$Tpz = Join-Path $CacheDir "Godot_v${Version}_export_templates.tpz"

if (-not (Test-Path $Tpz) -or $Force) {
    Write-Host "==> Downloading Godot $Version export templates (~700 MB)" -ForegroundColor Cyan
    Write-Host "    $Url"
    Invoke-WebRequest -Uri $Url -OutFile $Tpz -UseBasicParsing
}

$null = New-Item -ItemType Directory -Force -Path $TplDir
$TmpExtract = Join-Path $CacheDir "tpz-extract"
if (Test-Path $TmpExtract) { Remove-Item -Recurse -Force $TmpExtract }
$null = New-Item -ItemType Directory -Force -Path $TmpExtract

Write-Host "==> Extracting templates" -ForegroundColor Cyan
Expand-Archive -Path $Tpz -DestinationPath $TmpExtract -Force

# The tpz contains a single root folder "templates/" with all platform files inside.
$Source = Join-Path $TmpExtract "templates"
if (-not (Test-Path $Source)) {
    throw "Unexpected archive layout: $Source not found"
}
Get-ChildItem -Path $Source -File | ForEach-Object {
    Copy-Item -Path $_.FullName -Destination $TplDir -Force
}

Remove-Item -Recurse -Force $TmpExtract
Write-Host "==> Installed export templates into $TplDir" -ForegroundColor Green
