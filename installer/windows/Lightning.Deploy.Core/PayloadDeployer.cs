// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Deploy.Core;

using System.IO.Compression;

/// <summary>
/// Extracts the embedded payload zip into the install directory with full
/// rollback support. Re-running over a broken install is safe: existing
/// files are backed up, then replaced.
/// </summary>
public sealed class PayloadDeployer(RollbackJournal journal)
{
    public async Task<IReadOnlyList<string>> DeployAsync(
        Stream payloadZip,
        string installDirectory,
        IProgress<double>? progress,
        CancellationToken cancellationToken)
    {
        if (!Directory.Exists(installDirectory))
        {
            Directory.CreateDirectory(installDirectory);
            journal.RecordDirectoryCreated(installDirectory);
        }

        var deployed = new List<string>();
        using var archive = new ZipArchive(payloadZip, ZipArchiveMode.Read, leaveOpen: true);
        var total = archive.Entries.Count;
        var done = 0;

        foreach (var entry in archive.Entries)
        {
            cancellationToken.ThrowIfCancellationRequested();

            var targetPath = SafeJoin(installDirectory, entry.FullName);
            if (entry.FullName.EndsWith('/'))
            {
                if (!Directory.Exists(targetPath))
                {
                    Directory.CreateDirectory(targetPath);
                    journal.RecordDirectoryCreated(targetPath);
                }
                continue;
            }

            var parent = Path.GetDirectoryName(targetPath);
            if (parent is not null && !Directory.Exists(parent))
            {
                Directory.CreateDirectory(parent);
                journal.RecordDirectoryCreated(parent);
            }

            if (File.Exists(targetPath))
            {
                var backup = targetPath + ".bak";
                File.Copy(targetPath, backup, overwrite: true);
                journal.RecordFileReplaced(targetPath, backup);
            }
            else
            {
                journal.RecordFileCreated(targetPath);
            }

            await using (var source = entry.Open())
            await using (var target = File.Create(targetPath))
            {
                await source.CopyToAsync(target, cancellationToken);
            }

            deployed.Add(targetPath);
            done++;
            progress?.Report((double)done / total);
        }

        return deployed;
    }

    /// <summary>Confine extraction against zip-slip: the resolved path must
    /// stay inside the install directory.</summary>
    internal static string SafeJoin(string root, string relative)
    {
        var combined = Path.GetFullPath(Path.Combine(root, relative));
        var normalizedRoot = Path.GetFullPath(root + Path.DirectorySeparatorChar);
        if (!combined.StartsWith(normalizedRoot, StringComparison.OrdinalIgnoreCase))
        {
            throw new InvalidDataException($"payload entry escapes the install dir: {relative}");
        }

        return combined;
    }
}
