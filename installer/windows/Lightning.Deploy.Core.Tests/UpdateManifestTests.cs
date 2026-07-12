// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Deploy.Core.Tests;

using Xunit;

public sealed class UpdateManifestTests
{
    private static UpdateManifest Manifest(string version) => new()
    {
        Version = version,
        Channel = "stable",
        PublishedAt = DateTimeOffset.UtcNow,
        DownloadUrl = new Uri("https://github.com/neramc/lightning/releases/x.zip"),
        Sha256 = "00",
    };

    [Theory]
    [InlineData("1.2.0", "1.1.9", true)]
    [InlineData("1.1.9", "1.2.0", false)] // downgrade rejected
    [InlineData("1.2.0", "1.2.0", false)] // same version is not an update
    [InlineData("1.2.0", "1.2.0-beta.1", true)] // release beats its prerelease
    [InlineData("1.2.0-beta.1", "1.2.0", false)]
    [InlineData("1.2.0-beta.2", "1.2.0-beta.1", true)]
    public void Version_monotonicity(string candidate, string installed, bool expected) =>
        Assert.Equal(expected, Manifest(candidate).IsNewerThan(installed));

    [Fact]
    public void Manifest_round_trips_from_json()
    {
        const string Json = """
            {
              "version": "1.0.1",
              "channel": "beta",
              "pub_date": "2026-07-11T00:00:00Z",
              "url": "https://github.com/neramc/lightning/releases/download/v1.0.1/lightning.zip",
              "sha256": "abc123",
              "signature": null,
              "notes": "Lightning 1.0.1"
            }
            """;
        var manifest = UpdateManifest.FromJson(Json);
        Assert.Equal("1.0.1", manifest.Version);
        Assert.Equal("beta", manifest.Channel);
        Assert.True(manifest.IsNewerThan("1.0.0"));
    }
}
