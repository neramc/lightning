// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Installer;

using Microsoft.UI.Xaml;

/// <summary>Code-behind is wiring only (§10) — everything lives in the VM.</summary>
public partial class App : Application
{
    private Window? _window;

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        _window = new MainWindow();
        _window.Activate();
    }
}
