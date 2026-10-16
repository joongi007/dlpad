# Read-only native Windows development environment evidence. No GUI or installation.
$ErrorActionPreference = 'Stop'
$toolNames = @('rustup.exe', 'cargo.exe', 'rustc.exe', 'cl.exe', 'clang.exe', 'clang-cl.exe', 'cmake.exe', 'ninja.exe')
$tools = @{}
foreach ($name in $toolNames) {
    $command = Get-Command $name -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -eq $command) { $tools[$name] = $null } else { $tools[$name] = $command.Source }
}
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$result = [ordered]@{
    windowsVersion = [Environment]::OSVersion.Version.ToString()
    processArchitecture = if ([Environment]::Is64BitProcess) { 'x64' } else { 'x86' }
    tools = $tools
    userRustupExists = Test-Path (Join-Path $env:USERPROFILE '.rustup')
    userCargoBinExists = Test-Path (Join-Path $env:USERPROFILE '.cargo\bin')
    vswhereExists = Test-Path $vswhere
    projectDriveExists = Test-Path 'D:\Project\dlpad'
    tempAvailable = Test-Path ([IO.Path]::GetTempPath())
}
$result | ConvertTo-Json -Depth 4 -Compress
