#!/usr/bin/env pwsh
#Requires -Version 5.1

param(
    [string]$Dir = "",
    [switch]$Help
)

$Repo = "foksiny/discordscript"
$Binary = "discordscript"

function Write-Banner {
    @"

  ____  _           ____                      _       _
 |  _ \(_)___  ___ / ___|  ___ _ __ ___  ___(_)_ __ | |_
 | | | | / __|/ __|\___ \ / __| '__/ _ \/ __| | '_ \| __|
 | |_| | \__ \ (__  ___) | (__| | |  __/ (__| | |_) | |_
 |____/|_|___/\___||____/ \___|_|  \___|\___|_| .__/ \__|
                                              |_|

"@
    Write-Host "DiscordScript Installer for Windows" -ForegroundColor Cyan
    Write-Host "===================================" -ForegroundColor Cyan
    ""
}

function Show-Help {
    Write-Banner
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  powershell -c `"irm https://raw.githubusercontent.com/$Repo/main/scripts/install.ps1 | iex`""
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Dir <path>   Install to a custom directory (default: user's PATH directory)"
    Write-Host "  -Help         Show this help message"
    Write-Host ""
    Write-Host "Requirements:"
    Write-Host "  - Windows 7+ / PowerShell 5.1+"
    Write-Host "  - Rust (optional, for cargo install)"
    exit 0
}

function Install-ViaCargo {
    Write-Host "Installing via cargo..." -ForegroundColor Yellow
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) {
        return $false
    }
    Write-Host "Running: cargo install --git https://github.com/$Repo --force" -ForegroundColor Gray
    cargo install --git "https://github.com/$Repo" --force
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "Installed successfully!" -ForegroundColor Green
        Write-Host "Run '$Binary --help' to get started." -ForegroundColor Green
        return $true
    }
    return $false
}

function Install-ViaDownload {
    param([string]$InstallDir)

    $arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
    $platform = "${arch}-windows"
    $releaseUrl = "https://github.com/$Repo/releases/latest/download/${Binary}-${platform}.zip"

    Write-Host "Downloading $Binary for $platform..." -ForegroundColor Yellow

    $tmpDir = Join-Path $env:TEMP "discordscript-install-$(Get-Random)"
    New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
    $zipPath = Join-Path $tmpDir "${Binary}.zip"

    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $wc = New-Object System.Net.WebClient
        $wc.DownloadFile($releaseUrl, $zipPath)

        Write-Host "Extracting..." -ForegroundColor Yellow
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
        }

        $exePath = Join-Path $tmpDir "${Binary}.exe"
        $destPath = Join-Path $InstallDir "${Binary}.exe"
        Move-Item -Force -Path $exePath -Destination $destPath

        Write-Host ""
        Write-Host "Installed to $destPath" -ForegroundColor Green
        Write-Host "Run '$Binary --help' to get started." -ForegroundColor Green

        # Add to PATH if not already there
        $currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
        if ($currentPath -notlike "*$InstallDir*") {
            $newPath = "$InstallDir;$currentPath"
            [Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::User)
            Write-Host ""
            Write-Host "Added $InstallDir to your PATH (user scope)." -ForegroundColor Cyan
            Write-Host "Restart your terminal or run:" -ForegroundColor Cyan
            Write-Host "  `$env:Path = [Environment]::GetEnvironmentVariable('Path', 'User') + ';' + [Environment]::GetEnvironmentVariable('Path', 'Machine')" -ForegroundColor Cyan
        }
    }
    catch {
        Write-Host "Download failed: $_" -ForegroundColor Red
        Write-Host "Pre-built binaries may not be available yet. Please use 'cargo install' instead." -ForegroundColor Yellow
        return $false
    }
    finally {
        Remove-Item -Recurse -Force -Path $tmpDir -ErrorAction SilentlyContinue
    }
    return $true
}

# Main
if ($Help) { Show-Help }

Write-Banner

if (Test-Path ".git" -and (Test-Path "Cargo.toml")) {
    Write-Host "Detected project source directory." -ForegroundColor Cyan
    Write-Host "Building from source..."
    cargo build --release
    if ($LASTEXITCODE -eq 0) {
        $src = "target\release\${Binary}.exe"
        if ($Dir) {
            Copy-Item -Force -Path $src -Destination "$Dir\${Binary}.exe"
            Write-Host "Installed to $Dir\${Binary}.exe" -ForegroundColor Green
        } else {
            Write-Host "Binary built at $src" -ForegroundColor Green
            Write-Host "Add it to your PATH or copy it to a directory in your PATH." -ForegroundColor Yellow
        }
        exit 0
    }
    exit 1
}

if (Install-ViaCargo) {
    exit 0
}

$installDir = if ($Dir) { $Dir } else { "$env:USERPROFILE\.local\bin" }
Install-ViaDownload -InstallDir $installDir
