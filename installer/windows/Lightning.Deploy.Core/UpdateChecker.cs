// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Deploy.Core;

/// <summary>
/// Background update check against GitHub Releases (CLAUDE.md §6.10):
/// download the manifest, compare versions (downgrades rejected), download
/// the full package, verify minisign + SHA-256 before staging.
/// One shared HttpClient; CancellationToken flows through all IO (§10).
/// </summary>
public sealed class UpdateChecker(HttpClient httpClient, SignatureVerifier verifier)
{
    public async Task<UpdateManifest?> CheckAsync(
        Uri manifestUrl,
        string installedVersion,
        CancellationToken cancellationToken)
    {
        using var response = await httpClient.GetAsync(manifestUrl, cancellationToken);
        response.EnsureSuccessStatusCode();
        var json = await response.Content.ReadAsStringAsync(cancellationToken);
        var manifest = UpdateManifest.FromJson(json);
        return manifest.IsNewerThan(installedVersion) ? manifest : null;
    }

    /// <summary>
    /// Download to the staging directory and verify. The staged package is
    /// applied on next app start with rollback; the previous payload is kept
    /// until the first successful launch.
    /// </summary>
    public async Task<string> DownloadAndVerifyAsync(
        UpdateManifest manifest,
        string stagingDirectory,
        CancellationToken cancellationToken)
    {
        Directory.CreateDirectory(stagingDirectory);
        var fileName = Path.GetFileName(manifest.DownloadUrl.LocalPath);
        var stagedPath = Path.Combine(stagingDirectory, fileName);

        await using (var response = await httpClient.GetStreamAsync(
            manifest.DownloadUrl,
            cancellationToken))
        await using (var target = File.Create(stagedPath))
        {
            await response.CopyToAsync(target, cancellationToken);
        }

        // Hash first (cheap), then signature — both must pass before staging
        // is considered valid. Never weakened, even for tests (§17.4).
        var actualSha = await SignatureVerifier.ComputeSha256Async(
            stagedPath,
            cancellationToken);
        if (!actualSha.Equals(manifest.Sha256, StringComparison.OrdinalIgnoreCase))
        {
            File.Delete(stagedPath);
            throw new InvalidDataException("update package hash mismatch");
        }

        if (manifest.Signature is null
            || !await verifier.VerifyMinisignAsync(
                stagedPath,
                manifest.Signature,
                cancellationToken))
        {
            File.Delete(stagedPath);
            throw new InvalidDataException("update package signature invalid");
        }

        return stagedPath;
    }
}
