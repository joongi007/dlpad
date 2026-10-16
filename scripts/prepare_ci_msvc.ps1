# CI-only: original Visual Studio installation and registry remain unchanged.
$ErrorActionPreference = 'Stop'
if (-not $env:GITHUB_ENV -or -not $env:RUNNER_TEMP) { throw 'GitHub runner environment required' }
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $installation -or @($installation).Count -ne 1) { throw 'One installed x64 MSVC required' }
$env:VSCMD_SKIP_SENDTELEMETRY = '1'
# Microsoft-recommended build automation entry point; no composed cmd.exe strings.
& (Join-Path $installation 'Common7/Tools/Launch-VsDevShell.ps1') -Arch amd64 -HostArch amd64 -SkipAutomaticLocation
if (-not $env:VCToolsInstallDir -or -not $env:INCLUDE -or -not $env:LIB) { throw 'Incomplete VS compiler environment' }
$original = [IO.Path]::GetFullPath((Join-Path $env:VCToolsInstallDir 'bin/Hostx64/x64'))
$private = Join-Path $env:RUNNER_TEMP 'dlpad-ci-msvc-x64'
& python "$PSScriptRoot/prepare_ci_msvc.py" --source $original --destination $private --manifest ci-msvc-copy.json
if ($LASTEXITCODE -ne 0) { throw 'CI-private compiler preparation failed' }
$env:PATH = $private + ';' + (($env:PATH -split ';' | Where-Object { $_.TrimEnd('\') -ine $original.TrimEnd('\') }) -join ';')
$values = @{
    CC_x86_64_pc_windows_msvc = (Join-Path $private 'cl.exe')
    CXX_x86_64_pc_windows_msvc = (Join-Path $private 'cl.exe')
    AR_x86_64_pc_windows_msvc = (Join-Path $private 'lib.exe')
    CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = (Join-Path $private 'link.exe')
    INCLUDE = $env:INCLUDE
    LIB = $env:LIB
    LIBPATH = $env:LIBPATH
    VCINSTALLDIR = $env:VCINSTALLDIR
    VCToolsInstallDir = $env:VCToolsInstallDir
    VSCMD_ARG_TGT_ARCH = $env:VSCMD_ARG_TGT_ARCH
    PATH = $env:PATH
    VSCMD_SKIP_SENDTELEMETRY = '1'
}
foreach ($entry in $values.GetEnumerator()) {
    if ($entry.Value -match "[\r\n]") { throw 'Multiline tool environment rejected' }
    "$($entry.Key)=$($entry.Value)" | Out-File -FilePath $env:GITHUB_ENV -Encoding utf8 -Append
}
