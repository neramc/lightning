// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Installer;

using Lightning.Installer.ViewModels;
using Microsoft.UI.Xaml;

/// <summary>Wiring only — logic lives in <see cref="InstallerViewModel"/>.</summary>
public sealed partial class MainWindow : Window
{
    public InstallerViewModel ViewModel { get; } = new();

    public MainWindow()
    {
        InitializeComponent();
    }
}
