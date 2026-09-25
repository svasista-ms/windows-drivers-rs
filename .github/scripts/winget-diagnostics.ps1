param(
    [Parameter(Mandatory)]
    [ValidateSet('Snapshot', 'Probe', 'Collect', 'Intervene', 'StartTrace', 'StopTrace')]
    [string]$Phase,
    [string]$Label = 'before',
    [ValidateSet('control', 'register', 'upgrade', 'matching-module')]
    [string]$Experiment = 'control',
    [string]$ModuleVersion = '1.29.380',
    [string]$UpgradeTag = 'v1.29.380',
    [string]$ExpectedCliVersion = '1.11.510',
    [string]$OutputDirectory = "$env:RUNNER_TEMP/winget-diagnostics"
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

function Save-Diagnostic {
    param([string]$Name, [scriptblock]$Operation)
    try {
        & $Operation | Out-String -Width 240 | Set-Content "$OutputDirectory/$Label-$Name.txt"
    } catch {
        $_ | Format-List * -Force | Out-String -Width 240 | Set-Content "$OutputDirectory/$Label-$Name-error.txt"
        Write-Warning "$Name capture failed: $($_.Exception.Message)"
    }
}

function Get-CliVersion {
    $version = & winget --version
    if ($LASTEXITCODE -ne 0) { throw "winget --version exited with $LASTEXITCODE" }
    return ($version | Out-String).Trim().TrimStart('v')
}

function Save-ActivationState {
    param([string]$Prefix = '')
    Save-Diagnostic "${Prefix}processes" {
        Get-CimInstance Win32_Process -Filter "Name = 'WindowsPackageManagerServer.exe' OR Name = 'winget.exe'" |
            Select-Object Name, ProcessId, ParentProcessId, ExecutablePath, CommandLine, CreationDate | Format-List
    }
    Save-Diagnostic "${Prefix}pipes" {
        $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        "Expected endpoint: \\pipe\WinGetServerManualActivation_$sid"
        [IO.Directory]::GetFiles('\\.\pipe\') | Where-Object { $_ -match 'WinGet' }
    }
}

function Invoke-Probe {
    param([string]$Name, [scriptblock]$Operation)
    $started = [DateTime]::UtcNow
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $result = [ordered]@{ name = $Name; startedUtc = $started.ToString('o'); success = $false; errors = @() }
    try {
        & $Operation *>&1 | Out-File "$OutputDirectory/$Label-$Name.txt" -Width 240
        $result.success = $true
    } catch {
        $_ | Format-List * -Force | Out-String -Width 240 | Set-Content "$OutputDirectory/$Label-$Name-error.txt"
        $exception = $_.Exception
        while ($null -ne $exception) {
            $result.errors += [ordered]@{
                type = $exception.GetType().FullName
                hresult = ('0x{0:X8}' -f $exception.HResult)
                message = $exception.Message
                stack = $exception.StackTrace
            }
            $exception = $exception.InnerException
        }
        Write-Warning "$Name failed: $($_.Exception.Message)"
    }
    $timer.Stop()
    $result.elapsedSeconds = $timer.Elapsed.TotalSeconds
    $result | ConvertTo-Json -Depth 12 | Set-Content "$OutputDirectory/$Label-$Name-result.json"
    if ($env:GITHUB_STEP_SUMMARY) {
        "| $Label | $Name | $($result.success) | $([Math]::Round($timer.Elapsed.TotalSeconds, 2)) |" |
            Add-Content $env:GITHUB_STEP_SUMMARY
    }
}

switch ($Phase) {
    'Snapshot' {
        Save-Diagnostic 'identity' {
            [pscustomobject]@{
                TimeUtc = [DateTime]::UtcNow.ToString('o')
                ImageOS = $env:ImageOS
                ImageVersion = $env:ImageVersion
                RunId = $env:GITHUB_RUN_ID
                RunAttempt = $env:GITHUB_RUN_ATTEMPT
                Commit = $env:GITHUB_SHA
                RunnerOS = $env:RUNNER_OS
                RunnerArch = $env:RUNNER_ARCH
                PowerShell = $PSVersionTable.PSVersion.ToString()
                ProcessArch = [Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture
                OS = [Runtime.InteropServices.RuntimeInformation]::OSDescription
                User = [Security.Principal.WindowsIdentity]::GetCurrent().Name
                Elevated = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
                PSModulePath = $env:PSModulePath
            } | Format-List
            whoami /user
        }
        Save-Diagnostic 'cli' {
            Get-Command winget -All | Format-List Name, Source, Version
            $actual = Get-CliVersion
            "Actual CLI: $actual; expected historical CLI: $ExpectedCliVersion"
            if ($Label -eq 'before') {
                $actual | Set-Content "$OutputDirectory/original-cli.txt"
                if ($actual -ne $ExpectedCliVersion) {
                    Write-Warning "Image drift: CLI $actual is not $ExpectedCliVersion. This is not the historical version pair."
                }
            }
            winget --info
        }
        Save-Diagnostic 'packages' {
            Get-AppxPackage -AllUsers | Where-Object { $_.Name -match 'DesktopAppInstaller|VCLibs|UI.Xaml|WindowsAppRuntime' } |
                Format-List Name, Version, Architecture, PackageFullName, PackageFamilyName, InstallLocation, Status, PackageUserInformation, Dependencies
        }
        Save-Diagnostic 'manifest' {
            $package = Get-AppxPackage -Name Microsoft.DesktopAppInstaller
            Get-Content (Join-Path $package.InstallLocation 'AppxManifest.xml') -Raw
        }
        Save-Diagnostic 'binaries' {
            $package = Get-AppxPackage -Name Microsoft.DesktopAppInstaller
            Get-ChildItem $package.InstallLocation -Filter '*.exe' | ForEach-Object {
                [pscustomobject]@{ Name = $_.Name; Version = $_.VersionInfo.FileVersion; SHA256 = (Get-FileHash $_.FullName).Hash }
            } | Format-Table -AutoSize
        }
        Save-Diagnostic 'modules' { Get-Module Microsoft.WinGet.Client -ListAvailable | Format-List Name, Version, Path }
        Save-Diagnostic 'services' { Get-Service RpcSs, DcomLaunch, AppXSvc, ClipSVC | Format-Table -AutoSize }
        Save-ActivationState
    }
    'Probe' {
        if ($Label -eq 'after' -and (Test-Path "$OutputDirectory/selected-module.txt")) {
            $ModuleVersion = (Get-Content "$OutputDirectory/selected-module.txt" -Raw).Trim()
        }
        $moduleReady = $false
        Invoke-Probe 'module-import' {
            Import-Module Microsoft.WinGet.Client -RequiredVersion $ModuleVersion -Force -ErrorAction Stop
            Get-Module Microsoft.WinGet.Client | Format-List Name, Version, Path
            $script:moduleReady = $true
        }
        if ($moduleReady) {
            Invoke-Probe 'module-query' {
                Get-WinGetPackage -Id LLVM.LLVM -Source winget -MatchOption Equals -ErrorAction Stop -Verbose |
                    Format-List *
            }
            Save-ActivationState -Prefix 'module-query-'
            Save-Diagnostic 'loaded-assemblies' {
                [AppDomain]::CurrentDomain.GetAssemblies() | Where-Object { $_.FullName -match 'WinGet|Management.Deployment|WinRT' } |
                    Select-Object FullName, Location | Format-List
            }
            Save-Diagnostic 'native-modules' {
                (Get-Process -Id $PID).Modules | Where-Object { $_.ModuleName -match 'winrtact|WinGet|Management.Deployment' } |
                    ForEach-Object {
                        [pscustomobject]@{
                            Path = $_.FileName
                            Version = $_.FileVersionInfo.FileVersion
                            SHA256 = (Get-FileHash $_.FileName).Hash
                        }
                    } | Format-List
            }
        }
        Invoke-Probe 'cli-query' {
            $output = & winget list --id LLVM.LLVM --exact --source winget --accept-source-agreements --disable-interactivity --verbose-logs 2>&1
            $exitCode = $LASTEXITCODE
            $output | Set-Content "$OutputDirectory/$Label-cli-output.txt"
            "Exit code: $exitCode" | Add-Content "$OutputDirectory/$Label-cli-output.txt"
            if ($exitCode -ne 0 -and $exitCode -ne -1978335212) {
                throw "winget list exited with $exitCode; see $Label-cli-output.txt"
            }
            $output
        }
    }
    'Intervene' {
        switch ($Experiment) {
            'control' { 'No intervention; repeat in a fresh PowerShell process.' | Set-Content "$OutputDirectory/intervention.txt" }
            'register' {
                $before = Get-CliVersion
                $package = Get-AppxPackage -Name Microsoft.DesktopAppInstaller
                if (-not $package) { throw 'App Installer is not registered for the current user.' }
                Add-AppxPackage -Register (Join-Path $package.InstallLocation 'AppxManifest.xml') -DisableDevelopmentMode -ForceApplicationShutdown
                $after = Get-CliVersion
                if ($before -ne $after) { throw "Version changed during registration: $before -> $after" }
                "Re-registered existing package $($package.PackageFullName); CLI remained $after. No files reinstalled." | Set-Content "$OutputDirectory/intervention.txt"
            }
            'upgrade' {
                $downloadDirectory = Join-Path $OutputDirectory 'release'
                New-Item -ItemType Directory -Path $downloadDirectory -Force | Out-Null
                gh release view $UpgradeTag --repo microsoft/winget-cli --json tagName,url,targetCommitish,assets |
                    Set-Content "$OutputDirectory/release.json"
                if ($LASTEXITCODE -ne 0) { throw "Release $UpgradeTag is unavailable." }
                gh release download $UpgradeTag --repo microsoft/winget-cli --pattern '*.msixbundle' --pattern 'DesktopAppInstaller_Dependencies.zip' --dir $downloadDirectory
                if ($LASTEXITCODE -ne 0) { throw 'Release download failed.' }
                Get-ChildItem $downloadDirectory -File | Get-FileHash | ConvertTo-Json | Set-Content "$OutputDirectory/release-hashes.json"
                $dependencyDirectory = Join-Path $downloadDirectory 'dependencies'
                Expand-Archive (Join-Path $downloadDirectory 'DesktopAppInstaller_Dependencies.zip') $dependencyDirectory
                $dependencies = @(Get-ChildItem $dependencyDirectory -Recurse -File | Where-Object {
                    $_.Extension -in @('.appx', '.msix') -and $_.FullName -match '[\\/]x64[\\/]'
                })
                foreach ($dependency in $dependencies) {
                    try { Add-AppxPackage -Path $dependency.FullName -ForceApplicationShutdown }
                    catch {
                        if ($_.Exception.Message -notmatch '0x80073D06') { throw }
                        Write-Warning "Keeping newer dependency: $($dependency.Name)"
                    }
                }
                $bundle = @(Get-ChildItem $downloadDirectory -Filter '*.msixbundle')
                if ($bundle.Count -ne 1) { throw 'Expected exactly one release bundle.' }
                Add-AppxPackage -Path $bundle[0].FullName -ForceApplicationShutdown -ForceUpdateFromAnyVersion
                $actual = Get-CliVersion
                if ($actual -ne $UpgradeTag.TrimStart('v')) { throw "Requested $UpgradeTag, got $actual" }
                "Installed $UpgradeTag, including its framework dependencies. Module unchanged." | Set-Content "$OutputDirectory/intervention.txt"
                Remove-Item $downloadDirectory -Recurse -Force
            }
            'matching-module' {
                $version = Get-CliVersion
                $found = Find-PSResource Microsoft.WinGet.Client -Version $version -Repository PSGallery
                if (-not $found) { throw "No exact module $version exists: experiment unavailable; no fallback used." }
                Install-PSResource Microsoft.WinGet.Client -Version $version -Repository PSGallery -TrustRepository -Quiet
                $version | Set-Content "$OutputDirectory/selected-module.txt"
                "Selected module $version for the next fresh PowerShell process; CLI unchanged." | Set-Content "$OutputDirectory/intervention.txt"
            }
        }
    }
    'StartTrace' {
        Save-Diagnostic 'trace-start' {
            logman start WinGetDiagnosticProcesses -p Microsoft-Windows-Kernel-Process 0x10 5 -o "$OutputDirectory/process.etl" -f bincirc -max 32 -ets
            if ($LASTEXITCODE -ne 0) { throw "Process trace unavailable: $LASTEXITCODE" }
            'started' | Set-Content "$OutputDirectory/trace-started.txt"
        }
    }
    'StopTrace' {
        if (Test-Path "$OutputDirectory/trace-started.txt") {
            Save-Diagnostic 'trace-stop' {
                logman stop WinGetDiagnosticProcesses -ets
                if ($LASTEXITCODE -ne 0) { throw "Process trace stop failed: $LASTEXITCODE" }
            }
        }
    }
    'Collect' {
        $startFile = Join-Path $OutputDirectory 'started-utc.txt'
        $start = if (Test-Path $startFile) { [datetime]::Parse((Get-Content $startFile -Raw)).ToLocalTime() } else { (Get-Date).AddHours(-1) }
        foreach ($log in @('Application', 'System', 'Microsoft-Windows-AppXDeploymentServer/Operational', 'Microsoft-Windows-AppModel-Runtime/Admin', 'Microsoft-Windows-CodeIntegrity/Operational')) {
            $safeName = $log -replace '[/\\]', '-'
            Save-Diagnostic $safeName {
                Get-WinEvent -FilterHashtable @{ LogName = $log; StartTime = $start } -MaxEvents 500 |
                    Select-Object TimeCreated, Id, LevelDisplayName, ProviderName, Message | Format-List
            }
        }
        foreach ($root in @(
            "$env:LOCALAPPDATA/Packages/Microsoft.DesktopAppInstaller_8wekyb3d8bbwe/LocalState/DiagOutputDir",
            "$env:LOCALAPPDATA/Microsoft/WinGet/DiagOutputDir",
            "$env:TEMP/WinGet"
        )) {
            if (Test-Path $root) {
                $destination = Join-Path $OutputDirectory ("$Label-logs-" + [IO.Path]::GetFileName((Split-Path $root)))
                New-Item -ItemType Directory -Path $destination -Force | Out-Null
                Get-ChildItem $root -Recurse -File -Filter '*.log' | Where-Object { $_.LastWriteTime -ge $start } |
                    Copy-Item -Destination $destination -Force
            }
        }
    }
}