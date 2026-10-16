# Capability probe only: create/close a private desktop, never launch a GUI or switch desktops.
$ErrorActionPreference = 'Stop'
$probeTemp = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\.local-tools\windows-desktop-probe-tmp'))
[IO.Directory]::CreateDirectory($probeTemp) | Out-Null
$env:TEMP = $probeTemp
$env:TMP = $probeTemp
Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
public static class DlpadDesktopCapability {
    [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern IntPtr CreateDesktopW(string name, IntPtr device, IntPtr mode, uint flags, uint access, IntPtr security);
    [DllImport("user32.dll", SetLastError=true)]
    static extern bool CloseDesktop(IntPtr desktop);
    [DllImport("user32.dll", SetLastError=true)]
    static extern IntPtr OpenInputDesktop(uint flags, bool inherit, uint access);
    [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    static extern bool GetUserObjectInformationW(IntPtr handle, int index, StringBuilder name, uint size, out uint needed);
    delegate bool EnumWindow(IntPtr window, IntPtr parameter);
    [DllImport("user32.dll", SetLastError=true)]
    static extern bool EnumDesktopWindows(IntPtr desktop, EnumWindow callback, IntPtr parameter);
    static string InputName() {
        IntPtr handle = OpenInputDesktop(0, false, 1);
        if(handle == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        try {
            StringBuilder name = new StringBuilder(256);
            uint needed;
            if(!GetUserObjectInformationW(handle, 2, name, 512, out needed)) throw new Win32Exception(Marshal.GetLastWin32Error());
            return name.ToString();
        } finally { CloseDesktop(handle); }
    }
    public static string[] Probe() {
        string before = InputName();
        string privateName = "dlpad-probe-" + Guid.NewGuid().ToString("N");
        IntPtr desktop = CreateDesktopW(privateName, IntPtr.Zero, IntPtr.Zero, 0, 0x83, IntPtr.Zero);
        if(desktop == IntPtr.Zero) throw new Win32Exception(Marshal.GetLastWin32Error());
        int count = 0;
        try {
            EnumWindow callback = delegate(IntPtr window, IntPtr parameter) { count++; return true; };
            if(!EnumDesktopWindows(desktop, callback, IntPtr.Zero)) throw new Win32Exception(Marshal.GetLastWin32Error());
        } finally {
            if(!CloseDesktop(desktop)) throw new Win32Exception(Marshal.GetLastWin32Error());
        }
        string after = InputName();
        if(before != after) throw new InvalidOperationException("Input desktop changed during probe");
        return new string[] { before, after, count.ToString() };
    }
}
'@
$result = [DlpadDesktopCapability]::Probe()
[ordered]@{
    privateDesktopCreatedAndClosed = $true
    inputDesktopBefore = $result[0]
    inputDesktopAfter = $result[1]
    privateDesktopWindowCount = [int]$result[2]
    appGuiLaunched = $false
    switchedDesktop = $false
} | ConvertTo-Json -Compress
