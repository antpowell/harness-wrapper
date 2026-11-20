[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$wrapper = Join-Path $root 'dsh-webview-wrapper'
$launcher = Join-Path $root 'dsh-web-startup.bat'
$executable = Join-Path $wrapper 'src-tauri\target\release\dsh-webview-wrapper.exe'
$taskName = 'DeepSeekHarness Web'

& (Join-Path $wrapper 'build-wrapper.bat')
if ($LASTEXITCODE -ne 0) { throw 'The wrapper build failed.' }

if (-not (Test-Path $executable)) {
    throw "The release executable was not created: $executable"
}

$action = New-ScheduledTaskAction -Execute $launcher -WorkingDirectory $root
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew
$description = 'Starts DSH with --no-open and displays its authenticated endpoint in the native WebView2 wrapper.'

Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Settings $settings -Description $description -Force | Out-Null
Write-Host "Installed '$taskName' to run at logon: $launcher"

$startMenu = [Environment]::GetFolderPath('Programs')
$shortcutPath = Join-Path $startMenu 'DSH WebView.lnk'
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $launcher
$shortcut.WorkingDirectory = $root
$shortcut.IconLocation = $executable
$shortcut.WindowStyle = 7
$shortcut.Description = 'Start DeepSeek Harness in its native WebView2 window'
$shortcut.Save()
Write-Host "Created Start Menu shortcut: $shortcutPath"
