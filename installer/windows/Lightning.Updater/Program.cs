// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

using Lightning.Updater;

// `Lightning.Updater check|stage|apply` — invoked by the app on start and by
// a scheduled background check. Exit codes: 0 ok, 2 no update, 1 failure.
using var cancellation = new CancellationTokenSource();
Console.CancelKeyPress += (_, args) =>
{
    args.Cancel = true;
    cancellation.Cancel();
};

var command = args.Length > 0 ? args[0] : "check";
var service = new UpdateService();
try
{
    return command switch
    {
        "check" => await service.CheckAsync(cancellation.Token) ? 0 : 2,
        "stage" => await service.StageAsync(cancellation.Token) ? 0 : 2,
        "apply" => await service.ApplyStagedAsync(cancellation.Token) ? 0 : 2,
        _ => Fail($"unknown command '{command}'"),
    };
}
catch (OperationCanceledException)
{
    return 1;
}

static int Fail(string message)
{
    Console.Error.WriteLine(message);
    return 1;
}
