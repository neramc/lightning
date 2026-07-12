// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Capability slice — the probe snapshot; badges update live on
 *  capability://changed (§6.6). */

import { create } from 'zustand';

import {
  getCapabilities,
  onCapabilityChanged,
  type CapabilitySnapshotDto,
} from '@lightning/bindings';

/** Optimistic default while the first probe is in flight. */
export const EMPTY_SNAPSHOT: CapabilitySnapshotDto = {
  os: 'linux',
  environment: null,
  osLabel: 'Linux',
  capabilities: [],
};

interface CapabilitiesState {
  snapshot: CapabilitySnapshotDto;
  initialized: boolean;
  init: () => Promise<void>;
}

export const useCapabilitiesStore = create<CapabilitiesState>()((set, get) => ({
  snapshot: EMPTY_SNAPSHOT,
  initialized: false,

  init: async () => {
    if (get().initialized) return;
    set({ initialized: true });
    try {
      set({ snapshot: await getCapabilities() });
    } catch {
      // Outside a Tauri window (tests, storybook) the invoke fails; keep the default.
    }
    await onCapabilityChanged((snapshot) => set({ snapshot }));
  },
}));
