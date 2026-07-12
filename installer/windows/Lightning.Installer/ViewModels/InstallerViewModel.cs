// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Installer.ViewModels;

using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Lightning.Deploy.Core;

/// <summary>
/// Wizard state + the install flow (§6.10): payload extraction with rollback,
/// registry entries, uninstall entry. Per-user by default; per-machine is an
/// explicit elevated option. No async void, no .Result — MVVM commands are
/// async Tasks with a CancellationToken (§10).
/// </summary>
public sealed partial class InstallerViewModel : ObservableObject
{
    private readonly CancellationTokenSource _cancellation = new();

    [ObservableProperty]
    private string _installDirectory = DefaultInstallDirectory();

    [ObservableProperty]
    private bool _perMachine;

    [ObservableProperty]
    private bool _autostart = true;

    [ObservableProperty]
    private bool _addToPath;

    [ObservableProperty]
    private bool _fileAssociation = true;

    [ObservableProperty]
    private bool _protocolHandler = true;

    [ObservableProperty]
    private bool _startMenuShortcut = true;

    [ObservableProperty]
    private bool _desktopShortcut;

    [ObservableProperty]
    private double _progress;

    [ObservableProperty]
    private bool _isInstalling;

    private static string DefaultInstallDirectory() =>
        Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "Programs",
            "Lightning");

    [RelayCommand]
    private async Task InstallAsync()
    {
        if (IsInstalling)
        {
            return;
        }

        IsInstalling = true;
        var journal = new RollbackJournal();
        try
        {
            var deployer = new PayloadDeployer(journal);
            var progress = new Progress<double>(value => Progress = value);

            await using var payload = OpenEmbeddedPayload();
            var files = await deployer.DeployAsync(
                payload,
                InstallDirectory,
                progress,
                _cancellation.Token);

            var manifest = new InstallManifest
            {
                Version = typeof(InstallerViewModel).Assembly.GetName().Version?.ToString(3)
                    ?? "0.0.0",
                InstallDirectory = InstallDirectory,
                PerMachine = PerMachine,
                Options = new InstallOptions
                {
                    Autostart = Autostart,
                    AddToPath = AddToPath,
                    FileAssociation = FileAssociation,
                    ProtocolHandler = ProtocolHandler,
                    StartMenuShortcut = StartMenuShortcut,
                    DesktopShortcut = DesktopShortcut,
                },
                Files = files,
            };
            await manifest.SaveAsync(
                Path.Combine(InstallDirectory, "install-manifest.json"),
                _cancellation.Token);

            var appExe = Path.Combine(InstallDirectory, "Lightning.exe");
            var registry = new RegistryConfigurator(PerMachine);
            registry.WriteAppKey(manifest);
            registry.WriteUninstallEntry(
                manifest,
                Path.Combine(InstallDirectory, "Lightning.Uninstaller.exe"));
            if (FileAssociation)
            {
                registry.RegisterFileAssociation(appExe);
            }

            if (ProtocolHandler)
            {
                registry.RegisterProtocolHandler(appExe);
            }

            journal.Commit();
        }
        catch (OperationCanceledException)
        {
            journal.Rollback();
        }
        catch
        {
            journal.Rollback();
            throw;
        }
        finally
        {
            IsInstalling = false;
        }
    }

    [RelayCommand]
    private void Cancel() => _cancellation.Cancel();

    /// <summary>The payload zip is appended to the installer by release.yml;
    /// dev builds read it from beside the executable.</summary>
    private static FileStream OpenEmbeddedPayload()
    {
        var beside = Path.Combine(AppContext.BaseDirectory, "payload.zip");
        return File.OpenRead(beside);
    }
}
