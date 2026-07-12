// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Deploy.Core;

using System.Security.Cryptography;
using NSec.Cryptography;

/// <summary>
/// minisign (ed25519) verification for updater packages (CLAUDE.md §6.10).
/// Only the PUBLIC key ships here — signing happens exclusively in CI (§2).
/// </summary>
public sealed class SignatureVerifier(byte[] minisignPublicKey)
{
    /// <summary>Hex-encoded SHA-256 of a file, streamed.</summary>
    public static async Task<string> ComputeSha256Async(
        string path,
        CancellationToken cancellationToken)
    {
        await using var stream = File.OpenRead(path);
        using var sha = SHA256.Create();
        var digest = await sha.ComputeHashAsync(stream, cancellationToken);
        return Convert.ToHexStringLower(digest);
    }

    /// <summary>
    /// Verify a minisign signature document against the file. Supports the
    /// prehashed mode ("ED": ed25519 over Blake2b-512 of the content) and the
    /// legacy mode ("Ed": ed25519 over the raw content).
    /// </summary>
    public async Task<bool> VerifyMinisignAsync(
        string path,
        string signatureDocument,
        CancellationToken cancellationToken)
    {
        // minisign sig document: comment line, base64(sig blob), trusted
        // comment lines. The blob is: 2-byte alg, 8-byte key id, 64-byte sig.
        var base64 = signatureDocument
            .Split('\n')
            .Select(line => line.Trim())
            .FirstOrDefault(line => line.Length > 0 && !line.StartsWith("untrusted comment", StringComparison.Ordinal)
                && !line.StartsWith("trusted comment", StringComparison.Ordinal));
        if (base64 is null)
        {
            return false;
        }

        byte[] blob;
        try
        {
            blob = Convert.FromBase64String(base64);
        }
        catch (FormatException)
        {
            return false;
        }

        if (blob.Length != 2 + 8 + 64)
        {
            return false;
        }

        var algorithm = System.Text.Encoding.ASCII.GetString(blob, 0, 2);
        var signature = blob[10..];

        byte[] message;
        if (algorithm == "ED")
        {
            message = await ComputeBlake2b512Async(path, cancellationToken);
        }
        else if (algorithm == "Ed")
        {
            message = await File.ReadAllBytesAsync(path, cancellationToken);
        }
        else
        {
            return false;
        }

        var ed25519 = NSec.Cryptography.SignatureAlgorithm.Ed25519;
        var publicKey = NSec.Cryptography.PublicKey.Import(
            ed25519,
            minisignPublicKey,
            KeyBlobFormat.RawPublicKey);
        return ed25519.Verify(publicKey, message, signature);
    }

    private static async Task<byte[]> ComputeBlake2b512Async(
        string path,
        CancellationToken cancellationToken)
    {
        var content = await File.ReadAllBytesAsync(path, cancellationToken);
        var blake2b = HashAlgorithm.Blake2b_512;
        return blake2b.Hash(content);
    }
}
