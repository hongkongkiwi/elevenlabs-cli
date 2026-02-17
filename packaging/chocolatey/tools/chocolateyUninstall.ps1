$ErrorActionPreference = 'Stop'

$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$binaryPath = Join-Path $toolsDir "elevenlabs-cli.exe"

if (Test-Path $binaryPath) {
  Remove-Item $binaryPath -Force
  Write-Host "elevenlabs-cli uninstalled successfully"
}