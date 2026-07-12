// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Button primitive — visible focus ring, keyboard-operable (§9.1). */

import type { ButtonHTMLAttributes } from 'react';

export type ButtonVariant = 'primary' | 'ghost' | 'danger';

const variantClasses: Record<ButtonVariant, string> = {
  primary: 'bg-sky-600 text-white hover:bg-sky-500 active:bg-sky-700',
  ghost:
    'bg-transparent text-zinc-700 hover:bg-zinc-500/10 dark:text-zinc-200 dark:hover:bg-zinc-400/10',
  danger: 'bg-rose-600 text-white hover:bg-rose-500 active:bg-rose-700',
};

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
}

export function Button({ variant = 'primary', className = '', ...rest }: ButtonProps) {
  return (
    <button
      className={`inline-flex items-center gap-2 rounded-[10px] px-3 py-1.5 text-sm font-medium transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-500 disabled:opacity-50 ${variantClasses[variant]} ${className}`}
      {...rest}
    />
  );
}
