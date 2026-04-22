import { useEffect, useRef } from 'react';
import { EditIcon, MessageSquareIcon, XCircleIcon } from 'lucide-react';
import { cn } from '@/lib/cn';

export interface AgentContextMenuAction {
  id: 'rename' | 'focus-chat' | 'terminate';
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  destructive?: boolean;
}

const ACTIONS: AgentContextMenuAction[] = [
  { id: 'focus-chat', label: 'Focus chat', icon: MessageSquareIcon },
  { id: 'rename', label: 'Rename', icon: EditIcon },
  { id: 'terminate', label: 'Terminate', icon: XCircleIcon, destructive: true },
];

interface Props {
  x: number;
  y: number;
  onSelect: (id: AgentContextMenuAction['id']) => void;
  onClose: () => void;
}

export function AgentContextMenu({ x, y, onSelect, onClose }: Props): JSX.Element {
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('mousedown', onDocClick);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('mousedown', onDocClick);
      document.removeEventListener('keydown', onKey);
    };
  }, [onClose]);

  return (
    <div
      ref={ref}
      role="menu"
      aria-label="Agent actions"
      className={cn(
        'absolute z-50 min-w-[160px] rounded-panel border border-border bg-elevated p-1',
        'shadow-card',
      )}
      style={{ left: x, top: y }}
    >
      {ACTIONS.map((a) => (
        <button
          key={a.id}
          type="button"
          role="menuitem"
          onClick={() => {
            onSelect(a.id);
            onClose();
          }}
          className={cn(
            'flex w-full items-center gap-2 rounded-input px-2 py-1.5 text-left text-13',
            a.destructive
              ? 'text-status-error hover:bg-status-error/10'
              : 'text-text-primary hover:bg-hover',
          )}
        >
          <a.icon className="h-3 w-3 opacity-80" />
          <span>{a.label}</span>
        </button>
      ))}
    </div>
  );
}
