<#
.SYNOPSIS
Runs the Windows A1 acceptance smoke test against a built Qiongli Lite MCP artifact.

.DESCRIPTION
Starts the supplied qiongli-literature-provider.exe twice over JSON-RPC stdio. The
first process persists an isolated canary credential and the second verifies the
redacted provider status. The script then checks the persisted config and confirms
that its DACL is protected and contains one current-user FullControl rule.

The temporary config is always removed and every modified process environment
variable is restored. Console output is intentionally limited to a fixed pass/fail
message. Use EvidencePath to write a path-free, secret-free JSON receipt.

.PARAMETER BinaryPath
Path to the built qiongli-literature-provider.exe release artifact.

.PARAMETER EvidencePath
Optional destination for a machine-readable JSON acceptance receipt.

.PARAMETER TimeoutSeconds
Maximum time allowed for each stdio invocation. Defaults to 15 seconds.

.EXAMPLE
./tooling/scripts/windows_a1_acceptance.ps1 `
  -BinaryPath "$env:RUNNER_TEMP/qiongli-lite-mcp/qiongli-literature-provider.exe" `
  -EvidencePath "$env:RUNNER_TEMP/qiongli-windows-a1.json"
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$BinaryPath,

    [Parameter(Mandatory = $false)]
    [string]$EvidencePath = "",

    [Parameter(Mandatory = $false)]
    [ValidateRange(1, 120)]
    [int]$TimeoutSeconds = 15
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-AcceptanceCheck {
    param(
        [Parameter(Mandatory = $true)]
        [bool]$Condition,

        [Parameter(Mandatory = $true)]
        [string]$FailureCode
    )

    if (-not $Condition) {
        throw [System.InvalidOperationException]::new($FailureCode)
    }
}

function Get-RequiredJsonProperty {
    param(
        [Parameter(Mandatory = $true)]
        [object]$InputObject,

        [Parameter(Mandatory = $true)]
        [string]$Name,

        [Parameter(Mandatory = $true)]
        [string]$FailureCode
    )

    $property = $InputObject.PSObject.Properties[$Name]
    Assert-AcceptanceCheck -Condition ($null -ne $property) -FailureCode $FailureCode
    return $property.Value
}

function ConvertFrom-SingleJsonResponse {
    param(
        [Parameter(Mandatory = $true)]
        [AllowEmptyString()]
        [string]$Output,

        [Parameter(Mandatory = $true)]
        [string]$FailureCode
    )

    $lines = @(
        $Output -split "`r?`n" |
            Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    )
    Assert-AcceptanceCheck -Condition ($lines.Count -eq 1) -FailureCode $FailureCode

    try {
        return ConvertFrom-Json -InputObject $lines[0] -ErrorAction Stop
    }
    catch {
        throw [System.InvalidOperationException]::new($FailureCode)
    }
}

function Invoke-QiongliStdio {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Executable,

        [Parameter(Mandatory = $true)]
        [string]$Request,

        [Parameter(Mandatory = $true)]
        [int]$Timeout
    )

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $Executable
    $startInfo.Arguments = "--transport stdio"
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardInput = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    $started = $false
    try {
        $started = $process.Start()
        Assert-AcceptanceCheck -Condition $started -FailureCode "process_start_failed"

        $stdoutTask = $process.StandardOutput.ReadToEndAsync()
        $stderrTask = $process.StandardError.ReadToEndAsync()
        $process.StandardInput.Write($Request)
        $process.StandardInput.Write("`n")
        $process.StandardInput.Close()

        $exited = $process.WaitForExit($Timeout * 1000)
        if (-not $exited) {
            try {
                $process.Kill()
                $process.WaitForExit()
            }
            catch {
                # The fixed timeout result remains authoritative even if termination races.
            }
            throw [System.TimeoutException]::new("process_timeout")
        }

        $stdout = $stdoutTask.GetAwaiter().GetResult()
        $stderr = $stderrTask.GetAwaiter().GetResult()
        return [pscustomobject]@{
            ExitCode = $process.ExitCode
            Stdout = $stdout
            Stderr = $stderr
        }
    }
    finally {
        if ($started) {
            try {
                if (-not $process.HasExited) {
                    $process.Kill()
                    $process.WaitForExit()
                }
            }
            catch {
                # Cleanup errors are surfaced by the isolated-directory check below.
            }
        }
        $process.Dispose()
    }
}

function Get-Sha256Hex {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $stream = $null
    $algorithm = $null
    try {
        $stream = [System.IO.File]::OpenRead($Path)
        $algorithm = [System.Security.Cryptography.SHA256]::Create()
        $digest = $algorithm.ComputeHash($stream)
        return ([System.BitConverter]::ToString($digest)).Replace("-", "").ToLowerInvariant()
    }
    finally {
        if ($null -ne $algorithm) {
            $algorithm.Dispose()
        }
        if ($null -ne $stream) {
            $stream.Dispose()
        }
    }
}

$knownFailureCodes = [System.Collections.Generic.HashSet[string]]::new(
    [System.StringComparer]::Ordinal
)
foreach ($code in @(
    "unsupported_platform",
    "artifact_invalid",
    "process_start_failed",
    "process_timeout",
    "save_process_failed",
    "save_response_invalid",
    "status_process_failed",
    "status_response_invalid",
    "canary_exposed",
    "config_not_persisted",
    "acl_not_protected",
    "acl_owner_invalid",
    "acl_rule_count_invalid",
    "acl_rule_invalid",
    "cleanup_failed",
    "unexpected_failure"
)) {
    $null = $knownFailureCodes.Add($code)
}

$environmentNames = @(
    "QIONGLI_CONFIG_HOME",
    "QIONGLI_SEMANTIC_SCHOLAR_API_KEY",
    "SEMANTIC_SCHOLAR_API_KEY",
    "S2_API_KEY",
    "QIONGLI_MCPB_SEMANTIC_SCHOLAR_API_KEY"
)
$originalEnvironment = @{}
foreach ($name in $environmentNames) {
    $originalEnvironment[$name] = [System.Environment]::GetEnvironmentVariable(
        $name,
        [System.EnvironmentVariableTarget]::Process
    )
}

$failureCode = $null
$testRoot = $null
$configPath = $null
$artifactHash = $null
$artifactSize = $null
$saveExitCode = $null
$statusExitCode = $null
$invocationCount = 0
$canaryRedacted = $false
$persistenceVerified = $false
$daclProtected = $false
$ownerVerified = $false
$ruleCountVerified = $false
$fullControlVerified = $false
$environmentRestored = $false
$temporaryConfigRemoved = $false

try {
    $runningOnWindows = [System.Environment]::OSVersion.Platform -eq [System.PlatformID]::Win32NT
    Assert-AcceptanceCheck -Condition $runningOnWindows -FailureCode "unsupported_platform"
    Assert-AcceptanceCheck `
        -Condition (-not [string]::IsNullOrWhiteSpace($BinaryPath)) `
        -FailureCode "artifact_invalid"

    $resolvedBinary = [System.IO.Path]::GetFullPath($BinaryPath)
    Assert-AcceptanceCheck `
        -Condition ([System.IO.File]::Exists($resolvedBinary)) `
        -FailureCode "artifact_invalid"
    Assert-AcceptanceCheck `
        -Condition ([System.IO.Path]::GetFileName($resolvedBinary) -ieq "qiongli-literature-provider.exe") `
        -FailureCode "artifact_invalid"

    $artifactHash = Get-Sha256Hex -Path $resolvedBinary
    $artifactSize = ([System.IO.FileInfo]::new($resolvedBinary)).Length

    $testRoot = [System.IO.Path]::Combine(
        [System.IO.Path]::GetTempPath(),
        "qiongli-a1-" + [System.Guid]::NewGuid().ToString("N")
    )
    $null = [System.IO.Directory]::CreateDirectory($testRoot)
    $configPath = [System.IO.Path]::Combine($testRoot, "providers.json")

    [System.Environment]::SetEnvironmentVariable(
        "QIONGLI_CONFIG_HOME",
        $testRoot,
        [System.EnvironmentVariableTarget]::Process
    )
    foreach ($name in $environmentNames | Where-Object { $_ -ne "QIONGLI_CONFIG_HOME" }) {
        [System.Environment]::SetEnvironmentVariable(
            $name,
            $null,
            [System.EnvironmentVariableTarget]::Process
        )
    }

    $canary = "qiongli-a1-canary-" + [System.Guid]::NewGuid().ToString("N")
    $saveRequest = [ordered]@{
        jsonrpc = "2.0"
        id = 1
        method = "tools/call"
        params = [ordered]@{
            name = "qiongli_save_provider_config"
            arguments = [ordered]@{
                provider = "semantic-scholar"
                field = "api-key"
                value = $canary
            }
        }
    } | ConvertTo-Json -Depth 8 -Compress

    $invocationCount += 1
    $saveRun = Invoke-QiongliStdio `
        -Executable $resolvedBinary `
        -Request $saveRequest `
        -Timeout $TimeoutSeconds
    $saveExitCode = $saveRun.ExitCode
    Assert-AcceptanceCheck `
        -Condition ($saveRun.ExitCode -eq 0) `
        -FailureCode "save_process_failed"
    Assert-AcceptanceCheck `
        -Condition (
            $saveRun.Stdout.IndexOf($canary, [System.StringComparison]::Ordinal) -lt 0 -and
            $saveRun.Stderr.IndexOf($canary, [System.StringComparison]::Ordinal) -lt 0
        ) `
        -FailureCode "canary_exposed"

    $saveResponse = ConvertFrom-SingleJsonResponse `
        -Output $saveRun.Stdout `
        -FailureCode "save_response_invalid"
    $saveJsonRpc = Get-RequiredJsonProperty `
        -InputObject $saveResponse `
        -Name "jsonrpc" `
        -FailureCode "save_response_invalid"
    $saveResponseId = Get-RequiredJsonProperty `
        -InputObject $saveResponse `
        -Name "id" `
        -FailureCode "save_response_invalid"
    Assert-AcceptanceCheck `
        -Condition ($saveJsonRpc -ceq "2.0" -and $saveResponseId -eq 1) `
        -FailureCode "save_response_invalid"
    $saveResult = Get-RequiredJsonProperty `
        -InputObject $saveResponse `
        -Name "result" `
        -FailureCode "save_response_invalid"
    $saveContent = Get-RequiredJsonProperty `
        -InputObject $saveResult `
        -Name "structuredContent" `
        -FailureCode "save_response_invalid"
    $saveStatus = Get-RequiredJsonProperty $saveContent "status" "save_response_invalid"
    $saveProvider = Get-RequiredJsonProperty $saveContent "provider" "save_response_invalid"
    $saveField = Get-RequiredJsonProperty $saveContent "field" "save_response_invalid"
    $saveFlag = Get-RequiredJsonProperty $saveContent "saved" "save_response_invalid"
    $saveConfigPath = Get-RequiredJsonProperty `
        $saveContent `
        "config_path" `
        "save_response_invalid"
    Assert-AcceptanceCheck `
        -Condition (
            $saveStatus -ceq "saved" -and
            $saveProvider -ceq "semantic_scholar" -and
            $saveField -ceq "api_key" -and
            $saveFlag -eq $true -and
            [System.StringComparer]::OrdinalIgnoreCase.Equals(
                [System.IO.Path]::GetFullPath([string]$saveConfigPath),
                $configPath
            )
        ) `
        -FailureCode "save_response_invalid"

    $statusRequest = [ordered]@{
        jsonrpc = "2.0"
        id = 2
        method = "tools/call"
        params = [ordered]@{
            name = "qiongli_config_status"
            arguments = [ordered]@{}
        }
    } | ConvertTo-Json -Depth 8 -Compress

    $invocationCount += 1
    $statusRun = Invoke-QiongliStdio `
        -Executable $resolvedBinary `
        -Request $statusRequest `
        -Timeout $TimeoutSeconds
    $statusExitCode = $statusRun.ExitCode
    Assert-AcceptanceCheck `
        -Condition ($statusRun.ExitCode -eq 0) `
        -FailureCode "status_process_failed"
    Assert-AcceptanceCheck `
        -Condition (
            $statusRun.Stdout.IndexOf($canary, [System.StringComparison]::Ordinal) -lt 0 -and
            $statusRun.Stderr.IndexOf($canary, [System.StringComparison]::Ordinal) -lt 0
        ) `
        -FailureCode "canary_exposed"

    $statusResponse = ConvertFrom-SingleJsonResponse `
        -Output $statusRun.Stdout `
        -FailureCode "status_response_invalid"
    $statusJsonRpc = Get-RequiredJsonProperty `
        -InputObject $statusResponse `
        -Name "jsonrpc" `
        -FailureCode "status_response_invalid"
    $statusResponseId = Get-RequiredJsonProperty `
        -InputObject $statusResponse `
        -Name "id" `
        -FailureCode "status_response_invalid"
    Assert-AcceptanceCheck `
        -Condition ($statusJsonRpc -ceq "2.0" -and $statusResponseId -eq 2) `
        -FailureCode "status_response_invalid"
    $statusResult = Get-RequiredJsonProperty `
        -InputObject $statusResponse `
        -Name "result" `
        -FailureCode "status_response_invalid"
    $statusContent = Get-RequiredJsonProperty `
        -InputObject $statusResult `
        -Name "structuredContent" `
        -FailureCode "status_response_invalid"
    $providerStatuses = Get-RequiredJsonProperty `
        $statusContent `
        "providers" `
        "status_response_invalid"
    $semanticScholarStatus = Get-RequiredJsonProperty `
        $providerStatuses `
        "semantic_scholar" `
        "status_response_invalid"
    $redactedConfig = Get-RequiredJsonProperty `
        $statusContent `
        "redacted_config" `
        "status_response_invalid"
    $redactedProviders = Get-RequiredJsonProperty `
        $redactedConfig `
        "providers" `
        "status_response_invalid"
    $redactedSemanticScholar = Get-RequiredJsonProperty `
        $redactedProviders `
        "semantic_scholar" `
        "status_response_invalid"
    $redactedEnabled = Get-RequiredJsonProperty `
        $redactedSemanticScholar `
        "enabled" `
        "status_response_invalid"
    $redactedConfigured = Get-RequiredJsonProperty `
        $redactedSemanticScholar `
        "configured" `
        "status_response_invalid"
    $redactedFields = Get-RequiredJsonProperty `
        $redactedSemanticScholar `
        "fields" `
        "status_response_invalid"
    $redactedApiKey = Get-RequiredJsonProperty `
        $redactedFields `
        "api_key" `
        "status_response_invalid"
    Assert-AcceptanceCheck `
        -Condition (
            $semanticScholarStatus -ceq "configured" -and
            $redactedEnabled -eq $true -and
            $redactedConfigured -eq $true -and
            $redactedApiKey -ceq "configured"
        ) `
        -FailureCode "status_response_invalid"

    Assert-AcceptanceCheck `
        -Condition ([System.IO.File]::Exists($configPath)) `
        -FailureCode "config_not_persisted"
    try {
        $config = ConvertFrom-Json `
            -InputObject ([System.IO.File]::ReadAllText($configPath)) `
            -ErrorAction Stop
        $configVersion = Get-RequiredJsonProperty `
            $config `
            "version" `
            "config_not_persisted"
        $configProviders = Get-RequiredJsonProperty `
            $config `
            "providers" `
            "config_not_persisted"
        $configSemanticScholar = Get-RequiredJsonProperty `
            $configProviders `
            "semantic_scholar" `
            "config_not_persisted"
        $configEnabled = Get-RequiredJsonProperty `
            $configSemanticScholar `
            "enabled" `
            "config_not_persisted"
        $configApiKey = Get-RequiredJsonProperty `
            $configSemanticScholar `
            "api_key" `
            "config_not_persisted"
        Assert-AcceptanceCheck `
            -Condition (
                $configVersion -eq 1 -and
                $configEnabled -eq $true -and
                $configApiKey -ceq $canary
            ) `
            -FailureCode "config_not_persisted"
        $persistenceVerified = $true
    }
    catch {
        if ($_.Exception.Message -ceq "config_not_persisted") {
            throw
        }
        throw [System.InvalidOperationException]::new("config_not_persisted")
    }

    $acl = Get-Acl -LiteralPath $configPath -ErrorAction Stop
    $daclProtected = $acl.AreAccessRulesProtected -eq $true
    Assert-AcceptanceCheck `
        -Condition $daclProtected `
        -FailureCode "acl_not_protected"

    $currentUserSid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User
    $ownerSid = $acl.GetOwner([System.Security.Principal.SecurityIdentifier])
    $ownerVerified = (
        $null -ne $currentUserSid -and
        $null -ne $ownerSid -and
        $ownerSid.Value -ceq $currentUserSid.Value
    )
    Assert-AcceptanceCheck `
        -Condition $ownerVerified `
        -FailureCode "acl_owner_invalid"

    $rules = @(
        $acl.GetAccessRules(
            $true,
            $true,
            [System.Security.Principal.SecurityIdentifier]
        )
    )
    $ruleCountVerified = $rules.Count -eq 1
    Assert-AcceptanceCheck `
        -Condition $ruleCountVerified `
        -FailureCode "acl_rule_count_invalid"

    $rule = $rules[0]
    $fullControlChecks = @(
        $rule.IdentityReference.Value -ceq $currentUserSid.Value
        $rule.AccessControlType -eq [System.Security.AccessControl.AccessControlType]::Allow
        $rule.FileSystemRights -eq [System.Security.AccessControl.FileSystemRights]::FullControl
        $rule.InheritanceFlags -eq [System.Security.AccessControl.InheritanceFlags]::None
        $rule.PropagationFlags -eq [System.Security.AccessControl.PropagationFlags]::None
        $rule.IsInherited -eq $false
    )
    $fullControlVerified = $fullControlChecks -notcontains $false
    Assert-AcceptanceCheck `
        -Condition $fullControlVerified `
        -FailureCode "acl_rule_invalid"

    $canaryRedacted = $true
}
catch {
    $candidateCode = $_.Exception.Message
    if ($knownFailureCodes.Contains($candidateCode)) {
        $failureCode = $candidateCode
    }
    else {
        $failureCode = "unexpected_failure"
    }
}
finally {
    $environmentRestored = $true
    foreach ($name in $environmentNames) {
        try {
            [System.Environment]::SetEnvironmentVariable(
                $name,
                $originalEnvironment[$name],
                [System.EnvironmentVariableTarget]::Process
            )
            $restoredValue = [System.Environment]::GetEnvironmentVariable(
                $name,
                [System.EnvironmentVariableTarget]::Process
            )
            if (-not [object]::Equals($restoredValue, $originalEnvironment[$name])) {
                $environmentRestored = $false
            }
        }
        catch {
            $environmentRestored = $false
        }
    }

    if ($null -eq $testRoot) {
        $temporaryConfigRemoved = $true
    }
    else {
        try {
            if ([System.IO.Directory]::Exists($testRoot)) {
                [System.IO.Directory]::Delete($testRoot, $true)
            }
            $temporaryConfigRemoved = -not [System.IO.Directory]::Exists($testRoot)
        }
        catch {
            $temporaryConfigRemoved = $false
        }
    }
}

if (-not $environmentRestored -or -not $temporaryConfigRemoved) {
    if ($null -eq $failureCode) {
        $failureCode = "cleanup_failed"
    }
}

$passed = $null -eq $failureCode
$evidence = [ordered]@{
    schema_version = 1
    acceptance = "qiongli_windows_a1_release_artifact"
    status = if ($passed) { "passed" } else { "failed" }
    timestamp_utc = [System.DateTime]::UtcNow.ToString("o")
    failure_code = $failureCode
    source = [ordered]@{
        commit = [System.Environment]::GetEnvironmentVariable("GITHUB_SHA")
        run_id = [System.Environment]::GetEnvironmentVariable("GITHUB_RUN_ID")
        run_attempt = [System.Environment]::GetEnvironmentVariable("GITHUB_RUN_ATTEMPT")
    }
    artifact = [ordered]@{
        file_name = "qiongli-literature-provider.exe"
        sha256 = $artifactHash
        size_bytes = $artifactSize
    }
    execution = [ordered]@{
        transport = "stdio"
        invocation_count = $invocationCount
        save_exit_code = $saveExitCode
        status_exit_code = $statusExitCode
        canary_redacted = $canaryRedacted
    }
    persistence = [ordered]@{
        version = 1
        provider = "semantic_scholar"
        enabled = $persistenceVerified
        api_key_persisted = $persistenceVerified
    }
    acl = [ordered]@{
        protected = $daclProtected
        owner_is_current_user = $ownerVerified
        ace_count_is_one = $ruleCountVerified
        current_user_full_control_only = $fullControlVerified
    }
    cleanup = [ordered]@{
        environment_restored = $environmentRestored
        temporary_config_removed = $temporaryConfigRemoved
    }
    runtime = [ordered]@{
        platform = "windows"
        powershell_version = $PSVersionTable.PSVersion.ToString()
    }
}

if (-not [string]::IsNullOrWhiteSpace($EvidencePath)) {
    try {
        $resolvedEvidencePath = [System.IO.Path]::GetFullPath($EvidencePath)
        $evidenceDirectory = [System.IO.Path]::GetDirectoryName($resolvedEvidencePath)
        if (-not [string]::IsNullOrWhiteSpace($evidenceDirectory)) {
            $null = [System.IO.Directory]::CreateDirectory($evidenceDirectory)
        }
        $evidenceJson = $evidence | ConvertTo-Json -Depth 8
        [System.IO.File]::WriteAllText(
            $resolvedEvidencePath,
            $evidenceJson + [System.Environment]::NewLine,
            [System.Text.UTF8Encoding]::new($false)
        )
    }
    catch {
        if ($passed) {
            $passed = $false
            $failureCode = "unexpected_failure"
        }
    }
}

if ($passed) {
    Write-Output "Windows A1 acceptance: passed."
    exit 0
}

Write-Output "Windows A1 acceptance: failed ($failureCode)."
exit 1
