// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Uninstaller.ViewModels;

using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Lightning.Deploy.Core;

/// <summary>
/// Uninstall flow (§6.10): removes payload, registry entries, shortcuts,
/// protocol handlers. Asks whether to keep user data — **default: keep**
/// (§17.9: never delete user data without the keep-data prompt).
/// </summary>
public sealed partial class UninstallerViewModel : ObservableObject
{
    private readonly CancellationTokenSource _cancellation = new();

    /// <summary>Default keep — deleting shortcuts is opt-in.</summary>
    [ObservableProperty]
    private bool _keepUserData = true;

    [ObservableProperty]
    private bool _isWorking;

    [ObservableProperty]
    private bool _isDone;

    [RelayCommand]
    private async Task UninstallAsync()
    {
        if (IsWorking)
        {
            return;
        }

        IsWorking = true;
        try
        {
            var manifestPath = Path.Combine(AppContext.BaseDirectory, "install-manifest.json");
            var manifest = await InstallManifest.LoadAsync(manifestPath, _cancellation.Token);

            foreach (var file in manifest.Files)
            {
                if (File.Exists(file))
                {
                    File.Delete(file);
                }
            }

            new RegistryConfigurator(manifest.PerMachine).RemoveAll();

            if (!KeepUserData)
            {
                var dataDir = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                    "Lightning");
                if (Directory.Exists(dataDir))
                {
                    Directory.Delete(dataDir, recursive: true);
                }
            }

            IsDone = true;
        }
        finally
        {
            IsWorking = false;
        }
    }

    [RelayCommand]
    private void Cancel() => _cancellation.Cancel();
}
