[CmdletBinding()]
param(
    [ValidateSet('App', 'Nsis', 'Msi', 'All')]
    [string]$Mode = 'App',

    [switch]$SkipDependencyInstall,
    [switch]$SkipChecks
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot
$FrontendRoot = Join-Path $RepoRoot 'frontend'
$CargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
$LlvmBin = 'C:\Program Files\LLVM\bin'
$CmakeBin = 'C:\Program Files\CMake\bin'
$TargetTriple = 'x86_64-pc-windows-msvc'
$TauriConfigPath = Join-Path $FrontendRoot 'src-tauri\tauri.conf.json'

function Invoke-Checked {
    param(
        [Parameter(Mandatory)]
        [string]$Command,
        [Parameter()]
        [string[]]$Arguments = @(),
        [Parameter()]
        [string]$WorkingDirectory = $RepoRoot
    )

    Push-Location $WorkingDirectory
    try {
        & $Command @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "$Command failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}

function Assert-Command {
    param([Parameter(Mandatory)][string]$Name)

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' was not found. Open a new PowerShell terminal after installing prerequisites."
    }
}

$env:PATH = "$CargoBin;$LlvmBin;$CmakeBin;$env:PATH"
$env:LIBCLANG_PATH = $LlvmBin

foreach ($command in @('rustc', 'cargo', 'cmake', 'clang', 'pnpm')) {
    Assert-Command $command
}

$clangVersion = (& clang --version | Select-Object -First 1)
if ($clangVersion -notmatch '^clang version 18\.') {
    throw "Luna Telepresence currently requires LLVM/Clang 18.x for whisper-rs 0.13.2. Found: $clangVersion"
}

$rustHost = (& rustc -vV | Select-String '^host:\s+(.+)$').Matches.Groups[1].Value
if ($rustHost -ne $TargetTriple) {
    throw "Expected Rust host $TargetTriple but found $rustHost"
}

if (-not $SkipDependencyInstall) {
    Invoke-Checked -Command 'pnpm' -Arguments @('install', '--frozen-lockfile') -WorkingDirectory $FrontendRoot
}

Invoke-Checked -Command 'pnpm' -Arguments @('brand:apply') -WorkingDirectory $FrontendRoot

$TauriConfig = Get-Content $TauriConfigPath -Raw | ConvertFrom-Json
if ($null -eq $TauriConfig.plugins.updater) {
    throw "Post-brand tauri.conf.json must define plugins.updater because the Rust application registers tauri-plugin-updater."
}

if ($null -eq $TauriConfig.plugins.updater.pubkey) {
    throw "Post-brand tauri.conf.json plugins.updater must define pubkey (an empty value is valid while updates are disabled)."
}

Write-Host "Building $($TauriConfig.productName) $($TauriConfig.version) ($Mode)" -ForegroundColor Cyan
Write-Host "Repository: $RepoRoot"
Write-Host "LLVM: $clangVersion"

if (-not $SkipChecks) {
    Invoke-Checked -Command 'pnpm' -Arguments @('exec', 'tsc', '--noEmit') -WorkingDirectory $FrontendRoot
    Invoke-Checked -Command 'cargo' -Arguments @('check')
}

Invoke-Checked -Command 'cargo' -Arguments @('build', '--release', '-p', 'llama-helper')

$sidecarSource = Join-Path $RepoRoot 'target\release\llama-helper.exe'
$binariesDirectory = Join-Path $FrontendRoot 'src-tauri\binaries'
$sidecarDestination = Join-Path $binariesDirectory "llama-helper-$TargetTriple.exe"

if (-not (Test-Path -LiteralPath $sidecarSource)) {
    throw "Expected llama-helper output was not created: $sidecarSource"
}

New-Item -ItemType Directory -Path $binariesDirectory -Force | Out-Null
Copy-Item -LiteralPath $sidecarSource -Destination $sidecarDestination -Force

$tauriArguments = @('exec', 'tauri', 'build', '--no-sign')
switch ($Mode) {
    'App' { $tauriArguments += '--no-bundle' }
    'Nsis' { $tauriArguments += @('--bundles', 'nsis') }
    'Msi' { $tauriArguments += @('--bundles', 'msi') }
    'All' { $tauriArguments += @('--bundles', 'nsis,msi') }
}

Invoke-Checked -Command 'pnpm' -Arguments $tauriArguments -WorkingDirectory $FrontendRoot

$appPath = Join-Path $RepoRoot 'target\release\luna-telepresence.exe'
if (-not (Test-Path -LiteralPath $appPath)) {
    throw "Expected application executable was not created: $appPath"
}

Write-Host "`nBuild complete." -ForegroundColor Green
Write-Host "Application: $appPath"

if ($Mode -in @('Nsis', 'All')) {
    Get-ChildItem (Join-Path $RepoRoot 'target\release\bundle\nsis') -Filter '*.exe' |
        ForEach-Object { Write-Host "NSIS installer: $($_.FullName)" }
}

if ($Mode -in @('Msi', 'All')) {
    Get-ChildItem (Join-Path $RepoRoot 'target\release\bundle\msi') -Filter '*.msi' |
        ForEach-Object { Write-Host "MSI installer: $($_.FullName)" }
}
