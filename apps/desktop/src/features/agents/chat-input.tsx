import { useCallback, useEffect, useRef } from 'react';
import { SendHorizonalIcon } from 'lucide-react';
import { cn } from '@/lib/cn';

interface Props {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  onSend: (text: string) => void | Promise<void>;
  /** Key that triggers a focus-on-mount pass — used to refocus when
   *  switching agents so the draft is ready to continue. */
  focusKey?: string;
}

export function ChatInput({ value, onChange, disabled, onSend, focusKey }: Props): JSX.Element {
  const ref = useRef<HTMLTextAreaElement | null>(null);

  const send = useCallback(async () => {
    const trimmed = value.trim();
    if (!trimmed) return;
    onChange('');
    await onSend(trimmed);
  }, [value, onChange, onSend]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>): void => {
    const isMod = e.metaKey || e.ctrlKey;
    if (e.key === 'Enter' && isMod) {
      e.preventDefault();
      void send();
    }
  };

  // Autosize the textarea within [min, 180]px.
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = `${Math.min(el.scrollHeight, 180)}px`;
  }, [value]);

  // Focus on mount and whenever focusKey changes (i.e., agent switch).
  useEffect(() => {
    ref.current?.focus();
  }, [focusKey]);

  return (
    <div className="flex items-end gap-2 border-t border-border-subtle bg-panel px-4 py-3">
      <textarea
        ref={ref}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Message your agent… Cmd/Ctrl+Enter to send"
        disabled={disabled}
        rows={1}
        className={cn(
          'flex-1 resize-none rounded-input border border-border bg-elevated px-3 py-2',
          'text-13 text-text-primary placeholder:text-text-tertiary',
          'focus:border-accent focus:outline-none',
          'disabled:opacity-60',
        )}
      />
      <button
        type="button"
        onClick={() => void send()}
        disabled={disabled || value.trim().length === 0}
        className={cn(
          'flex h-10 w-10 items-center justify-center rounded-button bg-accent text-white',
          'hover:opacity-90 disabled:opacity-40',
        )}
        aria-label="Send"
      >
        <SendHorizonalIcon className="h-4 w-4" />
      </button>
    </div>
  );
}
