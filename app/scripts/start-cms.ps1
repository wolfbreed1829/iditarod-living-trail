param([switch]$Build)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
$dataDirectory = Join-Path $projectRoot '.cms/rust'
New-Item -ItemType Directory -Force -Path $dataDirectory | Out-Null
$credentials = Join-Path $dataDirectory 'credentials.env'
if (-not (Test-Path -LiteralPath $credentials)) {
    $random = New-Object byte[] 24
    $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
    $rng.GetBytes($random)
    $rng.Dispose()
    $password = [Convert]::ToBase64String($random)
    "TRAIL_ADMIN_USER=trail-admin`nTRAIL_ADMIN_PASSWORD=$password" | Set-Content -LiteralPath $credentials -Encoding utf8
    Write-Host "First-login credentials saved in $credentials"
}
foreach ($line in Get-Content -LiteralPath $credentials) {
    if ($line -match '^(TRAIL_ADMIN_USER|TRAIL_ADMIN_PASSWORD)=(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
    }
}
$env:TRAIL_CMS_ENABLED = '1'
$env:TRAIL_DATA_DIR = $dataDirectory
$env:TRAIL_ORIGIN = 'http://localhost:3107'
$env:LEPTOS_OUTPUT_NAME = 'living-trail'
$env:LEPTOS_SITE_ROOT = Join-Path $projectRoot 'target/site'
$env:LEPTOS_SITE_ADDR = '127.0.0.1:3107'
$env:PORT = '3107'
Remove-Item Env:TRAIL_BASE_PATH -ErrorAction SilentlyContinue
Remove-Item Env:TRAIL_CMS_URL -ErrorAction SilentlyContinue
Remove-Item Env:TRAIL_CMS_PUBLIC_URL -ErrorAction SilentlyContinue
if ($Build -or -not (Test-Path -LiteralPath 'target/release/living-trail.exe')) {
    cargo leptos build --release
    if ($LASTEXITCODE -ne 0) { throw 'Rust build failed.' }
}
Write-Host 'Trail Studio: http://localhost:3107/admin'
Write-Host 'Website: http://localhost:3107'
& (Join-Path $projectRoot 'target/release/living-trail.exe')
