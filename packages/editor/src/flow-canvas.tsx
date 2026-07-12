// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * The vertical block canvas (§9.1): dnd-kit sortable list. Blocks lift on
 * drag (scale 1.03 + shadow); siblings part with layout springs (§9.3).
 * Virtualization beyond 60 blocks is handled by the consuming view.
 */

import { useMemo } from 'react';
import { DndContext, closestCenter, type DragEndEvent } from '@dnd-kit/core';
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
  arrayMove,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useTranslation } from 'react-i18next';

import type { ActionDefDto, CapabilitySnapshotDto, StepDto } from '@lightning/bindings';
import { elevation } from '@lightning/ui';

import { ActionBlock } from './action-block';

export interface FlowCanvasProps {
  steps: StepDto[];
  catalog: ActionDefDto[];
  snapshot: CapabilitySnapshotDto;
  runStates?: Record<string, 'idle' | 'running' | 'success' | 'failed'>;
  onReorder: (steps: StepDto[]) => void;
  onConfigure?: (step: StepDto) => void;
}

function SortableBlock(props: {
  step: StepDto;
  def: ActionDefDto | undefined;
  snapshot: CapabilitySnapshotDto;
  runState: 'idle' | 'running' | 'success' | 'failed';
  onConfigure?: () => void;
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: props.step.uuid,
  });
  if (props.def === undefined) return null;
  return (
    <li
      ref={setNodeRef}
      {...attributes}
      {...listeners}
      style={{
        transform: CSS.Transform.toString(
          transform ? { ...transform, scaleX: isDragging ? 1.03 : 1, scaleY: isDragging ? 1.03 : 1 } : null,
        ),
        transition,
        boxShadow: isDragging ? elevation.lifted : undefined,
        zIndex: isDragging ? 10 : undefined,
      }}
      className="list-none"
    >
      <ActionBlock
        def={props.def}
        snapshot={props.snapshot}
        runState={props.runState}
        onConfigure={props.onConfigure}
      />
    </li>
  );
}

export function FlowCanvas({
  steps,
  catalog,
  snapshot,
  runStates = {},
  onReorder,
  onConfigure,
}: FlowCanvasProps) {
  const { t } = useTranslation('editor');
  const defsById = useMemo(() => new Map(catalog.map((def) => [def.id, def])), [catalog]);

  function handleDragEnd(event: DragEndEvent) {
    const { active, over } = event;
    if (over === null || active.id === over.id) return;
    const from = steps.findIndex((s) => s.uuid === active.id);
    const to = steps.findIndex((s) => s.uuid === over.id);
    if (from < 0 || to < 0) return;
    onReorder(arrayMove(steps, from, to));
  }

  if (steps.length === 0) {
    return (
      <p className="rounded-2xl border-2 border-dashed border-zinc-300 p-10 text-center text-sm text-zinc-500 dark:border-zinc-700">
        {t('canvas.empty')}
      </p>
    );
  }

  return (
    <DndContext collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
      <SortableContext items={steps.map((s) => s.uuid)} strategy={verticalListSortingStrategy}>
        <ol className="m-0 flex flex-col gap-2 p-0">
          {steps.map((step) => (
            <SortableBlock
              key={step.uuid}
              step={step}
              def={defsById.get(step.actionId)}
              snapshot={snapshot}
              runState={runStates[step.uuid] ?? 'idle'}
              onConfigure={onConfigure ? () => onConfigure(step) : undefined}
            />
          ))}
        </ol>
      </SortableContext>
    </DndContext>
  );
}
