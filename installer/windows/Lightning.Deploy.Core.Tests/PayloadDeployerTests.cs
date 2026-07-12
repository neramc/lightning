// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Deploy.Core.Tests;

using System.IO.Compression;
using Xunit;

public sealed class PayloadDeployerTests : IDisposable
{
    private readonly string _root = Directory.CreateTempSubdirectory("lightning-test-").FullName;

    public void Dispose()
    {
        if (Directory.Exists(_root))
        {
            Directory.Delete(_root, recursive: true);
        }
    }

    private static MemoryStream MakeZip(params (string Name, string Content)[] entries)
    {
        var stream = new MemoryStream();
        using (var archive = new ZipArchive(stream, ZipArchiveMode.Create, leaveOpen: true))
        {
            foreach (var (name, content) in entries)
            {
                var entry = archive.CreateEntry(name);
                using var writer = new StreamWriter(entry.Open());
                writer.Write(content);
            }
        }

        stream.Position = 0;
        return stream;
    }

    [Fact]
    public async Task Deploys_and_rolls_back_cleanly()
    {
        var journal = new RollbackJournal();
        var deployer = new PayloadDeployer(journal);
        using var zip = MakeZip(("Lightning.exe", "binary"), ("resources/app.json", "{}"));

        var target = Path.Combine(_root, "install");
        var files = await deployer.DeployAsync(zip, target, progress: null, CancellationToken.None);

        Assert.Equal(2, files.Count);
        Assert.True(File.Exists(Path.Combine(target, "Lightning.exe")));

        journal.Rollback();
        Assert.False(File.Exists(Path.Combine(target, "Lightning.exe")));
    }

    [Fact]
    public void Zip_slip_entries_are_rejected()
    {
        Assert.Throws<InvalidDataException>(() =>
            PayloadDeployer.SafeJoin(_root, "..\\..\\evil.dll"));
    }

    [Fact]
    public async Task Rerunning_over_an_existing_install_replaces_files()
    {
        var target = Path.Combine(_root, "install");
        Directory.CreateDirectory(target);
        File.WriteAllText(Path.Combine(target, "Lightning.exe"), "broken-old");

        var deployer = new PayloadDeployer(new RollbackJournal());
        using var zip = MakeZip(("Lightning.exe", "fresh"));
        await deployer.DeployAsync(zip, target, progress: null, CancellationToken.None);

        Assert.Equal("fresh", File.ReadAllText(Path.Combine(target, "Lightning.exe")));
    }
}
