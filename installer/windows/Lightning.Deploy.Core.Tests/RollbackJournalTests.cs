// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Deploy.Core.Tests;

using Xunit;

public sealed class RollbackJournalTests : IDisposable
{
    private readonly string _root = Directory.CreateTempSubdirectory("lightning-test-").FullName;

    public void Dispose()
    {
        if (Directory.Exists(_root))
        {
            Directory.Delete(_root, recursive: true);
        }
    }

    [Fact]
    public void Rollback_deletes_created_files_and_restores_replaced_ones()
    {
        var journal = new RollbackJournal();

        var created = Path.Combine(_root, "new.dll");
        File.WriteAllText(created, "new");
        journal.RecordFileCreated(created);

        var replaced = Path.Combine(_root, "app.exe");
        File.WriteAllText(replaced, "old-contents");
        var backup = replaced + ".bak";
        File.Copy(replaced, backup);
        File.WriteAllText(replaced, "new-contents");
        journal.RecordFileReplaced(replaced, backup);

        journal.Rollback();

        Assert.False(File.Exists(created));
        Assert.Equal("old-contents", File.ReadAllText(replaced));
        Assert.False(File.Exists(backup));
        Assert.Empty(journal.Entries);
    }

    [Fact]
    public void Rollback_survives_already_missing_targets()
    {
        var journal = new RollbackJournal();
        journal.RecordFileCreated(Path.Combine(_root, "never-existed.bin"));
        journal.Rollback(); // must not throw
        Assert.Empty(journal.Entries);
    }

    [Fact]
    public void Commit_clears_without_touching_files()
    {
        var journal = new RollbackJournal();
        var kept = Path.Combine(_root, "keep.txt");
        File.WriteAllText(kept, "x");
        journal.RecordFileCreated(kept);
        journal.Commit();
        Assert.True(File.Exists(kept));
        Assert.Empty(journal.Entries);
    }
}
