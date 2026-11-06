$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot
python scripts/backup-cms.py
if ($LASTEXITCODE -ne 0) { throw 'Backup failed.' }
