param(
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [string]$EndUtc,
 [ValidateRange(1,48)][int]$Hours=36
)
$ErrorActionPreference='Stop'
# Invoke only through the existing hidden Windows CLI host. This script queries
# fixed logs; it does not change power, WSL, desktop, service or task settings.
if(-not [IO.Path]::IsPathRooted($OutputDirectory)){throw 'Absolute fresh output required'}
$full=[IO.Path]::GetFullPath($OutputDirectory)
if($OutputDirectory.Split([char[]]@('\','/')) -contains '..'){throw 'Parent traversal rejected'}
for($cursor=$full;$cursor;$cursor=[IO.Path]::GetDirectoryName($cursor)){
 if(Test-Path -LiteralPath $cursor){
  if(([IO.File]::GetAttributes($cursor) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Reparse output rejected'}
 }
}
if(Test-Path -LiteralPath $full){throw 'Fresh output directory required'}
if(-not (Test-Path -LiteralPath ([IO.Path]::GetDirectoryName($full)) -PathType Container)){throw 'Output parent must exist'}
$end=if($EndUtc){[DateTimeOffset]::Parse($EndUtc,[Globalization.CultureInfo]::InvariantCulture).UtcDateTime}else{[DateTime]::UtcNow}
$start=$end.AddHours(-$Hours)
$systemProviders=@('Microsoft-Windows-Kernel-Power','Microsoft-Windows-Power-Troubleshooter','User32','EventLog')
$systemIds=@(1,41,42,107,109,506,507,1074,6005,6006,6008)
$pairs=@{
 'Microsoft-Windows-Kernel-Power'=@(41,42,107,109,506,507)
 'Microsoft-Windows-Power-Troubleshooter'=@(1)
 'User32'=@(1074)
 'EventLog'=@(6005,6006,6008)
}
$logs=@()
foreach($name in @('System','Microsoft-Windows-Lxss/Operational','Microsoft-Windows-Hyper-V-Compute-Admin')){
 $row=[ordered]@{log=$name;available=$false;registered=$false;enabled=$null;max_events=32;events=@();cause='unconfirmed'}
 try{
  $info=Get-WinEvent -ListLog $name -ErrorAction Stop
  $row.registered=$true;$row.enabled=[bool]$info.IsEnabled
  $filter=@{LogName=$name;StartTime=$start;EndTime=$end}
  if($name -eq 'System'){$filter.ProviderName=$systemProviders;$filter.Id=$systemIds}
  try{
   $events=@(Get-WinEvent -FilterHashtable $filter -MaxEvents 32 -ErrorAction Stop)
   $safe=@()
   foreach($event in $events){
    if($name -eq 'System' -and (-not $pairs.ContainsKey($event.ProviderName) -or $pairs[$event.ProviderName] -notcontains $event.Id)){continue}
    $safe+=@{utc=$event.TimeCreated.ToUniversalTime().ToString('o');id=[int]$event.Id;provider=[string]$event.ProviderName;level=$event.Level}
   }
   $row.events=$safe;$row.available=$true
  }catch{
   if($_.FullyQualifiedErrorId -like 'NoMatchingEventsFound*'){
    $row.available=$true;$row.events=@();$row.query_status='no-matching-events'
   }else{
    $row.query_status='query-failed';$row.error_type=$_.Exception.GetType().FullName;$row.error_hresult=$_.Exception.HResult
   }
  }
 }catch{
  $row.query_status='log-unavailable';$row.error_type=$_.Exception.GetType().FullName;$row.error_hresult=$_.Exception.HResult
 }
 $logs+=$row
}
$snapshot=[ordered]@{
 observed_utc=[DateTime]::UtcNow.ToString('o');start_utc=$start.ToString('o');end_utc=$end.ToString('o')
 scope='read-only fixed Windows logs; no messages, account identifiers or machine names'
 shutdown_requester='unconfirmed';causal_attribution='event presence alone does not identify who stopped WSL'
 host_watchdog_installed=$false;logs=$logs
}
$utf8=New-Object Text.UTF8Encoding($false)
$encoded=$utf8.GetBytes(($snapshot|ConvertTo-Json -Depth 8))
if($encoded.Length -gt 65536){throw 'Snapshot byte limit exceeded'}
$sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
$acl=New-Object Security.AccessControl.DirectorySecurity
$acl.SetOwner($sid);$acl.SetAccessRuleProtection($true,$false)
$acl.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow')))
(New-Object IO.DirectoryInfo($full)).Create($acl)
$file=Join-Path $full 'windows-events.json'
$stream=[IO.File]::Open($file,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None)
try{$stream.Write($encoded,0,$encoded.Length);$stream.Flush($true)}finally{$stream.Dispose()}
# No arbitrary exception messages or raw event objects are printed.
@{created=$file;bytes=$encoded.Length;logs=@($logs|ForEach-Object{@{log=$_.log;available=$_.available;registered=$_.registered;events=@($_.events).Count}})}|ConvertTo-Json -Depth 5
