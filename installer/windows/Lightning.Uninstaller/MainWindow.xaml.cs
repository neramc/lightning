// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

namespace Lightning.Uninstaller;

using Lightning.Uninstaller.ViewModels;
using Microsoft.UI.Xaml;

public sealed partial class MainWindow : Window
{
    public UninstallerViewModel ViewModel { get; } = new();

    public MainWindow()
    {
        InitializeComponent();
    }
}
