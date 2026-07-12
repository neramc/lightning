// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * Auto-rendered parameter UI (§6.5): the editor renders every action's
 * params from its schema — never a per-action hand-built form.
 */

import { useTranslation } from 'react-i18next';

import type { ParamDefDto } from '@lightning/bindings';
import { actionParamKey } from '@lightning/i18n';

export interface ParamFieldProps {
  actionId: string;
  def: ParamDefDto;
  value: string;
  onChange: (value: string) => void;
}

const inputClasses =
  'w-full rounded-[10px] border border-zinc-300 bg-white px-2 py-1 text-sm dark:border-zinc-700 dark:bg-zinc-900 focus-visible:outline focus-visible:outline-2 focus-visible:outline-sky-500';

export function ParamField({ actionId, def, value, onChange }: ParamFieldProps) {
  const { t } = useTranslation();
  const label = t(actionParamKey(actionId, def.key));
  const id = `${actionId}-${def.key}`;

  return (
    <label
      htmlFor={id}
      className="flex flex-col gap-1 text-xs font-medium text-zinc-600 dark:text-zinc-300"
    >
      {label}
      {renderControl(id, def, value, onChange)}
    </label>
  );
}

function renderControl(
  id: string,
  def: ParamDefDto,
  value: string,
  onChange: (value: string) => void,
) {
  switch (def.kind) {
    case 'boolean':
      return (
        <input
          id={id}
          type="checkbox"
          checked={value === 'true'}
          onChange={(e) => onChange(String(e.target.checked))}
          className="size-4 accent-sky-600"
        />
      );
    case 'number':
      return (
        <input
          id={id}
          type="number"
          value={value}
          required={def.required}
          onChange={(e) => onChange(e.target.value)}
          className={inputClasses}
        />
      );
    case 'date':
      return (
        <input
          id={id}
          type="datetime-local"
          value={value}
          required={def.required}
          onChange={(e) => onChange(e.target.value)}
          className={inputClasses}
        />
      );
    case 'enum':
      return (
        <select
          id={id}
          value={value}
          required={def.required}
          onChange={(e) => onChange(e.target.value)}
          className={inputClasses}
        >
          {(def.options ?? []).map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      );
    case 'file':
    case 'text':
    default:
      return (
        <input
          id={id}
          type="text"
          value={value}
          required={def.required}
          onChange={(e) => onChange(e.target.value)}
          className={inputClasses}
        />
      );
  }
}
