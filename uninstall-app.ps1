[CmdletBinding()]
param(
    [switch]$Machine
)

$ErrorActionPreference = 'Stop'

if ($Machine) {
    $installDir = Join-Path $env:ProgramFiles 'DSH WebView'
    $startMenuDir = Join-Path $env:ProgramData 'Microsoft\Windows\Start Menu\Programs'
    $regPath = 'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\DSHWebView'
} else {
    $installDir = Join-Path $env:LOCALAPPDATA 'DSH WebView'
    $startMenuDir = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs'
    $regPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\DSHWebView'
}

$shortcutPath = Join-Path $startMenuDir 'DSH WebView.lnk'

if (Test-Path $shortcutPath) {
    Remove-Item $shortcutPath -Force
}

if (Test-Path $installDir) {
    Remove-Item $installDir -Recurse -Force
}

if (Test-Path $regPath) {
    Remove-Item $regPath -Recurse -Force
}

Write-Host 'DSH WebView has been uninstalled.'
