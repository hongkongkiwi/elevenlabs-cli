$ErrorActionPreference = 'Stop'

$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$version = '__VERSION__'
$packageName = 'elevenlabs-cli'

# Download URL for Windows binary
$url = "https://github.com/hongkongkiwi/elevenlabs-cli/releases/download/v$version/elevenlabs-cli-v$version-x86_64-pc-windows-msvc.zip"

# Download and extract
$packageArgs = @{
  packageName   = $packageName
  url           = $url
  unzipLocation = $toolsDir
  checksum      = '__SHA256__'
  checksumType  = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

# Make the binary executable
$binaryPath = Join-Path $toolsDir "elevenlabs-cli.exe"
if (Test-Path $binaryPath) {
  Write-Host "elevenlabs-cli installed successfully to $binaryPath"
} else {
  Write-Error "Failed to install elevenlabs-cli"
}