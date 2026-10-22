param(
    [Parameter(Position=0)]
    [string]$Target,
    [Parameter(Position=1)]
    [string]$Inputs,
    [Parameter(Position=2)]
    [string]$Output = "golden_master.json"
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

if (Get-Command node -ErrorAction SilentlyContinue) {
    node "$scriptDir\generate_golden_master.js" $Target $Inputs $Output
    exit $LASTEXITCODE
} elseif (Get-Command php -ErrorAction SilentlyContinue) {
    php "$scriptDir\generate_golden_master.php" $Target $Inputs $Output
    exit $LASTEXITCODE
} else {
    Write-Error "Neither node nor php found on PATH"
    exit 1
}
