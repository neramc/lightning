// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Deploy.Core;

/// <summary>
/// Records every filesystem mutation so a failed install can be undone and
/// a re-run over a broken install stays safe (CLAUDE.md §10: registry/file
/// operations are idempotent and rollback-safe).
/// </summary>
public sealed class RollbackJournal
{
    private readonly List<JournalEntry> _entries = [];

    public IReadOnlyList<JournalEntry> Entries => _entries;

    public void RecordFileCreated(string path) =>
        _entries.Add(new JournalEntry(JournalAction.DeleteFile, path, null));

    public void RecordFileReplaced(string path, string backupPath) =>
        _entries.Add(new JournalEntry(JournalAction.RestoreFile, path, backupPath));

    public void RecordDirectoryCreated(string path) =>
        _entries.Add(new JournalEntry(JournalAction.DeleteDirectory, path, null));

    /// <summary>Undo everything, newest first. Never throws on missing
    /// targets — rollback must survive partial state.</summary>
    public void Rollback()
    {
        for (var i = _entries.Count - 1; i >= 0; i--)
        {
            var entry = _entries[i];
            try
            {
                switch (entry.Action)
                {
                    case JournalAction.DeleteFile when File.Exists(entry.Path):
                        File.Delete(entry.Path);
                        break;
                    case JournalAction.RestoreFile when entry.BackupPath is not null
                        && File.Exists(entry.BackupPath):
                        File.Copy(entry.BackupPath, entry.Path, overwrite: true);
                        File.Delete(entry.BackupPath);
                        break;
                    case JournalAction.DeleteDirectory when Directory.Exists(entry.Path)
                        && Directory.GetFileSystemEntries(entry.Path).Length == 0:
                        Directory.Delete(entry.Path);
                        break;
                }
            }
            catch (IOException)
            {
                // Best effort: keep unwinding the remaining entries.
            }
            catch (UnauthorizedAccessException)
            {
                // Same — a locked file must not abort the rest of the rollback.
            }
        }

        _entries.Clear();
    }

    public void Commit() => _entries.Clear();
}

public enum JournalAction
{
    DeleteFile,
    RestoreFile,
    DeleteDirectory,
}

public sealed record JournalEntry(JournalAction Action, string Path, string? BackupPath);
