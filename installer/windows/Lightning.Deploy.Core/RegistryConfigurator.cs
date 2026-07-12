// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Deploy.Core;

using System.Runtime.Versioning;
using Microsoft.Win32;

/// <summary>
/// Registry writes for install/uninstall (CLAUDE.md §6.10): app key under
/// <c>HKCU\Software\neramc\Lightning</c> (or HKLM per-machine), the uninstall
/// entry, the <c>.lightning</c> association, and the <c>lightning://</c>
/// protocol. Every operation is idempotent — re-running over a broken
/// install must succeed.
/// </summary>
[SupportedOSPlatform("windows")]
public sealed class RegistryConfigurator(bool perMachine)
{
    private const string AppKeyPath = @"Software\neramc\Lightning";
    private const string UninstallKeyPath =
        @"Software\Microsoft\Windows\CurrentVersion\Uninstall\Lightning";
    private const string ProgId = "Lightning.lightning";

    private RegistryKey Root => perMachine ? Registry.LocalMachine : Registry.CurrentUser;

    public void WriteAppKey(InstallManifest manifest)
    {
        using var key = Root.CreateSubKey(AppKeyPath);
        key.SetValue("InstallDir", manifest.InstallDirectory);
        key.SetValue("Version", manifest.Version);
    }

    public void WriteUninstallEntry(InstallManifest manifest, string uninstallerPath)
    {
        using var key = Root.CreateSubKey(UninstallKeyPath);
        key.SetValue("DisplayName", "Lightning");
        key.SetValue("DisplayVersion", manifest.Version);
        key.SetValue("Publisher", "neramc");
        key.SetValue("InstallLocation", manifest.InstallDirectory);
        key.SetValue("UninstallString", $"\"{uninstallerPath}\"");
        key.SetValue("DisplayIcon", Path.Combine(manifest.InstallDirectory, "Lightning.exe"));
        key.SetValue("NoModify", 1, RegistryValueKind.DWord);
        key.SetValue("NoRepair", 1, RegistryValueKind.DWord);
        // GPL-3.0 — the license URL is user-visible in Apps & Features.
        key.SetValue("URLInfoAbout", "https://github.com/neramc/lightning");
    }

    public void RegisterFileAssociation(string appExePath)
    {
        using (var ext = Root.CreateSubKey(@"Software\Classes\.lightning"))
        {
            ext.SetValue(string.Empty, ProgId);
        }

        using var progId = Root.CreateSubKey($@"Software\Classes\{ProgId}");
        progId.SetValue(string.Empty, "Lightning Shortcut");
        using (var icon = progId.CreateSubKey("DefaultIcon"))
        {
            icon.SetValue(string.Empty, $"\"{appExePath}\",0");
        }

        using var command = progId.CreateSubKey(@"shell\open\command");
        // Imports never auto-run — the app opens the permission review (§14).
        command.SetValue(string.Empty, $"\"{appExePath}\" \"%1\"");
    }

    public void RegisterProtocolHandler(string appExePath)
    {
        using var key = Root.CreateSubKey(@"Software\Classes\lightning");
        key.SetValue(string.Empty, "URL:Lightning deep link");
        key.SetValue("URL Protocol", string.Empty);
        using var command = key.CreateSubKey(@"shell\open\command");
        command.SetValue(string.Empty, $"\"{appExePath}\" \"%1\"");
    }

    /// <summary>Remove everything this class may have written. Missing keys
    /// are fine — uninstall is idempotent too.</summary>
    public void RemoveAll()
    {
        Root.DeleteSubKeyTree(AppKeyPath, throwOnMissingSubKey: false);
        Root.DeleteSubKeyTree(UninstallKeyPath, throwOnMissingSubKey: false);
        Root.DeleteSubKeyTree(@"Software\Classes\.lightning", throwOnMissingSubKey: false);
        Root.DeleteSubKeyTree($@"Software\Classes\{ProgId}", throwOnMissingSubKey: false);
        Root.DeleteSubKeyTree(@"Software\Classes\lightning", throwOnMissingSubKey: false);
    }
}
