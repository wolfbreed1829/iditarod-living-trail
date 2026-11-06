$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
foreach ($line in Get-Content -LiteralPath '.cms/rust/credentials.env') {
    if ($line -match '^(TRAIL_ADMIN_USER|TRAIL_ADMIN_PASSWORD)=(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
    }
}
$env:TRAIL_CMS_ENABLED = '1'
$env:TRAIL_DATA_DIR = Join-Path $projectRoot '.cms/native-test'
$env:TRAIL_ORIGIN = 'http://localhost:3110'
$env:LEPTOS_OUTPUT_NAME = 'living-trail'
$env:LEPTOS_SITE_ROOT = Join-Path $projectRoot 'target/site'
$env:LEPTOS_SITE_ADDR = '127.0.0.1:3110'
$env:PORT = '3110'
Remove-Item Env:TRAIL_BASE_PATH -ErrorAction SilentlyContinue
& (Join-Path $projectRoot 'target/release/living-trail.exe')
