// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Deploy.Core;

using System.Text.Json;
using System.Text.Json.Serialization;

/// <summary>
/// The install manifest written next to the payload: what was installed,
/// where, and with which options — consumed by the uninstaller and updater.
/// </summary>
public sealed record InstallManifest
{
    [JsonPropertyName("version")]
    public required string Version { get; init; }

    [JsonPropertyName("installDir")]
    public required string InstallDirectory { get; init; }

    [JsonPropertyName("perMachine")]
    public bool PerMachine { get; init; }

    [JsonPropertyName("options")]
    public required InstallOptions Options { get; init; }

    [JsonPropertyName("files")]
    public IReadOnlyList<string> Files { get; init; } = [];

    private static readonly JsonSerializerOptions SerializerOptions = new()
    {
        WriteIndented = true,
    };

    public string ToJson() => JsonSerializer.Serialize(this, SerializerOptions);

    public static InstallManifest FromJson(string json) =>
        JsonSerializer.Deserialize<InstallManifest>(json, SerializerOptions)
        ?? throw new InvalidDataException("install manifest is empty");

    public static async Task<InstallManifest> LoadAsync(
        string path,
        CancellationToken cancellationToken)
    {
        await using var stream = File.OpenRead(path);
        return await JsonSerializer.DeserializeAsync<InstallManifest>(
                stream,
                SerializerOptions,
                cancellationToken)
            ?? throw new InvalidDataException($"install manifest is empty: {path}");
    }

    public async Task SaveAsync(string path, CancellationToken cancellationToken)
    {
        await using var stream = File.Create(path);
        await JsonSerializer.SerializeAsync(stream, this, SerializerOptions, cancellationToken);
    }
}

/// <summary>User-selected install options (CLAUDE.md §6.10).</summary>
public sealed record InstallOptions
{
    [JsonPropertyName("autostart")]
    public bool Autostart { get; init; }

    [JsonPropertyName("addToPath")]
    public bool AddToPath { get; init; }

    [JsonPropertyName("fileAssociation")]
    public bool FileAssociation { get; init; } = true;

    [JsonPropertyName("protocolHandler")]
    public bool ProtocolHandler { get; init; } = true;

    [JsonPropertyName("startMenuShortcut")]
    public bool StartMenuShortcut { get; init; } = true;

    [JsonPropertyName("desktopShortcut")]
    public bool DesktopShortcut { get; init; }
}
