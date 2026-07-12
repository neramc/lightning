// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Updater;

using System.Runtime.Versioning;
using Lightning.Deploy.Core;
using Microsoft.Win32;

/// <summary>
/// The updater flow (§6.10): check → download+verify → stage → apply on next
/// app start, with rollback (the previous payload is kept until the first
/// successful launch). Channels: stable · beta · nightly. Downgrades are
/// rejected by <see cref="UpdateManifest.IsNewerThan"/>.
/// </summary>
public sealed class UpdateService
{
    // One shared HttpClient for the process lifetime (§10).
    private static readonly HttpClient Http = new()
    {
        Timeout = TimeSpan.FromMinutes(5),
    };

    // The minisign PUBLIC key (safe to embed; the secret key lives only in
    // CI secrets — §2). Placeholder until the release key is generated.
    private static readonly byte[] MinisignPublicKey = new byte[32];

    private const string ManifestUrlTemplate =
        "https://github.com/neramc/lightning/releases/latest/download/latest-windows-{0}-{1}.json";

    public async Task<bool> CheckAsync(CancellationToken cancellationToken)
    {
        var (installedVersion, channel) = ReadInstalledState();
        var manifestUrl = new Uri(string.Format(
            System.Globalization.CultureInfo.InvariantCulture,
            ManifestUrlTemplate,
            ArchitectureId(),
            channel));
        var checker = new UpdateChecker(Http, new SignatureVerifier(MinisignPublicKey));
        var update = await checker.CheckAsync(manifestUrl, installedVersion, cancellationToken);
        return update is not null;
    }

    public async Task<bool> StageAsync(CancellationToken cancellationToken)
    {
        var (installedVersion, channel) = ReadInstalledState();
        var manifestUrl = new Uri(string.Format(
            System.Globalization.CultureInfo.InvariantCulture,
            ManifestUrlTemplate,
            ArchitectureId(),
            channel));
        var checker = new UpdateChecker(Http, new SignatureVerifier(MinisignPublicKey));
        var update = await checker.CheckAsync(manifestUrl, installedVersion, cancellationToken);
        if (update is null)
        {
            return false;
        }

        var stagingDir = Path.Combine(AppContext.BaseDirectory, "staged-update");
        await checker.DownloadAndVerifyAsync(update, stagingDir, cancellationToken);
        return true;
    }

    public async Task<bool> ApplyStagedAsync(CancellationToken cancellationToken)
    {
        var stagingDir = Path.Combine(AppContext.BaseDirectory, "staged-update");
        var staged = Directory.Exists(stagingDir)
            ? Directory.GetFiles(stagingDir, "*.zip").FirstOrDefault()
            : null;
        if (staged is null)
        {
            return false;
        }

        // Keep the previous payload for rollback until the app launches once.
        var journal = new RollbackJournal();
        var deployer = new PayloadDeployer(journal);
        try
        {
            await using var payload = File.OpenRead(staged);
            await deployer.DeployAsync(
                payload,
                AppContext.BaseDirectory,
                progress: null,
                cancellationToken);
            // The journal (with .bak backups) is committed by the app's
            // first successful launch; a crash before that triggers rollback.
            File.Delete(staged);
            return true;
        }
        catch
        {
            journal.Rollback();
            throw;
        }
    }

    private static (string Version, string Channel) ReadInstalledState()
    {
        if (OperatingSystem.IsWindows())
        {
            return ReadFromRegistry();
        }

        return ("0.0.0", "stable");
    }

    [SupportedOSPlatform("windows")]
    private static (string Version, string Channel) ReadFromRegistry()
    {
        using var key = Registry.CurrentUser.OpenSubKey(@"Software\neramc\Lightning");
        var version = key?.GetValue("Version") as string ?? "0.0.0";
        var channel = key?.GetValue("Channel") as string ?? "stable";
        return (version, channel);
    }

    private static string ArchitectureId() =>
        System.Runtime.InteropServices.RuntimeInformation.OSArchitecture switch
        {
            System.Runtime.InteropServices.Architecture.Arm64 => "aarch64",
            _ => "x86_64",
        };
}
