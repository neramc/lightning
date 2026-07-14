# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 neramc
# FreeBSD port — Tier 3, best-effort (CLAUDE.md §7).

PORTNAME=	lightning
DISTVERSIONPREFIX=	v
DISTVERSION=	0.1.0
CATEGORIES=	deskutils

MAINTAINER=	ports@example.invalid
COMMENT=	Shortcuts and automations for your desktop
WWW=		https://github.com/neramc/lightning

LICENSE=	GPLv3
LICENSE_FILE=	${WRKSRC}/LICENSE.md

BUILD_DEPENDS=	rust>=1.85:lang/rust \
		node22>=22:www/node22 \
		npm-node22>0:www/npm-node22
LIB_DEPENDS=	libwebkit2gtk-4.1.so:www/webkit2-gtk@41

USE_GITHUB=	yes
GH_ACCOUNT=	neramc

do-build:
	cd ${WRKSRC} && pnpm install --frozen-lockfile && pnpm build

do-install:
	${INSTALL_PROGRAM} \
		${WRKSRC}/target/release/lightning-desktop \
		${STAGEDIR}${PREFIX}/bin/lightning
	${INSTALL_DATA} ${WRKSRC}/packaging/linux/lightning.desktop \
		${STAGEDIR}${PREFIX}/share/applications/lightning.desktop

.include <bsd.port.mk>
