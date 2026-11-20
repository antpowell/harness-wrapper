[CmdletBinding()]
param(
    [switch]$Build,
    [switch]$Machine,
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$wrapperDir = Join-Path $root 'dsh-webview-wrapper'
$sourceExe = Join-Path $wrapperDir 'src-tauri\target\release\dsh-webview-wrapper.exe'

# Build a release binary unless explicitly skipped.
if ($Build -or (-not $SkipBuild -and -not (Test-Path $sourceExe))) {
    & (Join-Path $wrapperDir 'build-wrapper.bat')
    if ($LASTEXITCODE -ne 0) { throw 'The wrapper build failed.' }
}

if (-not (Test-Path $sourceExe)) {
    throw "The release executable was not found: $sourceExe"
}

# Decide between a per-user install (default) and a machine-wide install (requires admin).
if ($Machine) {
    $installDir = Join-Path $env:ProgramFiles 'DSH WebView'
    $startMenuDir = Join-Path $env:ProgramData 'Microsoft\Windows\Start Menu\Programs'
    $regRoot = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall'
} else {
    $installDir = Join-Path $env:LOCALAPPDATA 'DSH WebView'
    $startMenuDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
    $regRoot = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall'
}

New-Item -ItemType Directory -Force -Path $installDir | Out-Null

$destExe = Join-Path $installDir 'dsh-webview-wrapper.exe'
Copy-Item -Path $sourceExe -Destination $destExe -Force

# Create a Start Menu shortcut.
$shortcutPath = Join-Path $startMenuDir 'DSH WebView.lnk'
$WshShell = New-Object -ComObject WScript.Shell
$shortcut = $WshShell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $destExe
$shortcut.WorkingDirectory = $installDir
$shortcut.IconLocation = "$destExe,0"
$shortcut.Description = 'DeepSeek Harness WebView Wrapper'
$shortcut.Save()

# Register the app so it appears in Settings > Apps > Installed apps.
$regPath = Join-Path $regRoot 'DSHWebView'
New-Item -Path $regPath -Force | Out-Null
New-ItemProperty -Path $regPath -Name 'DisplayName' -Value 'DSH WebView' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $regPath -Name 'DisplayIcon' -Value $destExe -PropertyType String -Force | Out-Null
New-ItemProperty -Path $regPath -Name 'Publisher' -Value 'DSH Team' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $regPath -Name 'DisplayVersion' -Value '0.1.0' -PropertyType String -Force | Out-Null
New-ItemProperty -Path $regPath -Name 'InstallLocation' -Value $installDir -PropertyType String -Force | Out-Null
$uninstallArgs = if ($Machine) { ' -Machine' } else { '' }
New-ItemProperty -Path $regPath -Name 'UninstallString' -Value "powershell.exe -ExecutionPolicy Bypass -File `"$root\uninstall-app.ps1`"$uninstallArgs" -PropertyType String -Force | Out-Null

Write-Host "DSH WebView installed to: $installDir"
Write-Host "Start Menu shortcut: $shortcutPath"
