#!/usr/bin/env powershell

param(
    [switch]$Json
)

$ErrorActionPreference = 'Stop'

function Write-JsonReportAndExit {
    param(
        [string]$Status,
        [string]$Message,
        [int]$ExitCode,
        [string]$CodexPath = '',
        [string]$CodexVersion = '',
        [string]$LoginStatus = ''
    )

    if ($Json) {
        [ordered]@{
            status = $Status
            message = $Message
            os_version = [System.Environment]::OSVersion.VersionString
            architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
            codex_path = $CodexPath
            codex_version = $CodexVersion
            login_status = $LoginStatus
            token_files_read = $false
            token_files_written = $false
        } | ConvertTo-Json -Depth 3
        exit $ExitCode
    }

    if ($ExitCode -eq 0) {
        Write-Output $Message
    } else {
        Write-Error $Message
    }
    exit $ExitCode
}

$codex = Get-Command -Name codex -Type Application -ErrorAction SilentlyContinue
if (-not $codex) {
    Write-JsonReportAndExit `
        -Status 'error' `
        -Message 'Codex CLI was not found on PATH. Install Codex, then run `codex` or `codex login --device-auth`.' `
        -ExitCode 1
}

if (-not $Json) {
    Write-Output "Codex CLI: $($codex.Source)"
}
$version = & $codex.Source --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-JsonReportAndExit `
        -Status 'error' `
        -Message 'Codex CLI exists but `codex --version` failed.' `
        -ExitCode $LASTEXITCODE `
        -CodexPath $codex.Source `
        -CodexVersion ($version -join "`n")
}
if (-not $Json) {
    Write-Output $version
}

$loginStatus = & $codex.Source login status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-JsonReportAndExit `
        -Status 'error' `
        -Message "Unable to check Codex login status. Run `codex` or `codex login --device-auth`, then try again. Output: $loginStatus" `
        -ExitCode $LASTEXITCODE `
        -CodexPath $codex.Source `
        -CodexVersion ($version -join "`n") `
        -LoginStatus ($loginStatus -join "`n")
}

if (-not $Json) {
    Write-Output $loginStatus
}
if ($loginStatus -notmatch 'Logged in') {
    Write-JsonReportAndExit `
        -Status 'error' `
        -Message 'Codex CLI is not logged in. Run `codex` or `codex login --device-auth`, then try again.' `
        -ExitCode 1 `
        -CodexPath $codex.Source `
        -CodexVersion ($version -join "`n") `
        -LoginStatus ($loginStatus -join "`n")
}

Write-JsonReportAndExit `
    -Status 'ok' `
    -Message 'Codex CLI is installed and logged in.' `
    -ExitCode 0 `
    -CodexPath $codex.Source `
    -CodexVersion ($version -join "`n") `
    -LoginStatus ($loginStatus -join "`n")
