# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 neramc
# Tier 3 — community-maintained, manual CI (CLAUDE.md §7).

EAPI=8

DESCRIPTION="Shortcuts and automations for your desktop"
HOMEPAGE="https://github.com/neramc/lightning"
SRC_URI="https://github.com/neramc/lightning/archive/refs/tags/v${PV}.tar.gz -> ${P}.tar.gz"

LICENSE="GPL-3"
SLOT="0"
KEYWORDS="~amd64"

DEPEND="
	net-libs/webkit-gtk:4.1
	dev-libs/libayatana-appindicator
	dev-libs/openssl
"
RDEPEND="${DEPEND}"
BDEPEND="
	dev-lang/rust
	net-libs/nodejs
"

src_compile() {
	pnpm install --frozen-lockfile || die
	pnpm build || die
}

src_install() {
	dobin apps/desktop/src-tauri/target/release/lightning-desktop
	newbin apps/desktop/src-tauri/target/release/lightning-desktop lightning
	domenu packaging/linux/lightning.desktop
	doicon -s 512 resources/icon.png
}
