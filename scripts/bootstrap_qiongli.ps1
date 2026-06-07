$ErrorActionPreference = "Stop"
$Target = Join-Path (Split-Path -Parent $PSScriptRoot) "tooling/scripts/bootstrap_qiongli.ps1"
& $Target @args
exit $LASTEXITCODE
