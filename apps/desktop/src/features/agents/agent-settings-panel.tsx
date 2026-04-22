import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { cn } from '@/lib/cn';
import { useAgentsStore } from '@/stores/agents';
import { ipcAgentRename, ipcAgentTerminate } from '@/lib/ipc';
import type { Agent } from '@orbit/types';

/**
 * Right-panel Settings tab. Phase 2 ships the shell only: basic info,
 * rename, and a Terminate action. Soul / Purpose / Memory editors
 * arrive in Phase 3.
 */
export function AgentSettingsPanel(): JSX.Element {
  const agent: Agent | null = useAgentsStore((s) =>
    s.selectedAgentId ? (s.agents[s.selectedAgentId] ?? null) : null,
  );

  if (!agent) {
    return (
      <div className="flex h-full items-center justify-center text-13 text-text-tertiary">
        No agent selected.
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-y-auto p-4">
      <div className="flex items-center gap-3">
        <span
          className="flex h-10 w-10 items-center justify-center rounded-full"
          style={{ backgroundColor: `${agent.color}26` }}
        >
          <span className="orbit-emoji text-[22px] leading-none">{agent.emoji}</span>
        </span>
        <div className="flex flex-col">
          <span className="text-14 font-medium text-text-primary">{agent.name}</span>
          <span className="text-11 text-text-tertiary">{agent.workingDir}</span>
        </div>
      </div>

      <section className="mt-6 flex flex-col gap-4">
        <RenameRow agentId={agent.id} currentName={agent.name} />
        <InfoRow label="Status" value={agent.status} />
        <InfoRow label="Session" value={agent.sessionId ?? '—'} mono />
        <InfoRow label="Model" value={agent.modelOverride ?? 'default'} mono />
      </section>

      <section className="mt-6 flex flex-col gap-2 rounded-panel border border-border-subtle bg-elevated p-3 text-12 text-text-tertiary">
        <span className="font-medium text-text-secondary">Coming in later phases</span>
        <ul className="list-disc pl-4">
          <li>Soul / Purpose / Memory (Phase 3)</li>
          <li>Folder access (Phase 5)</li>
          <li>Team membership (Phase 5)</li>
          <li>Git worktree (Phase 6)</li>
        </ul>
      </section>

      <TerminateButton agentId={agent.id} />
    </div>
  );
}

function InfoRow({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}): JSX.Element {
  return (
    <div className="flex items-baseline justify-between gap-3">
      <span className="text-11 uppercase tracking-wider text-text-tertiary">{label}</span>
      <span
        className={cn(
          'truncate text-13 text-text-primary',
          mono && 'font-mono text-12 text-text-secondary',
        )}
      >
        {value}
      </span>
    </div>
  );
}

function RenameRow({
  agentId,
  currentName,
}: {
  agentId: string;
  currentName: string;
}): JSX.Element {
  const [draft, setDraft] = useState(currentName);
  const [error, setError] = useState<string | null>(null);
  const renameAgent = useAgentsStore((s) => s.renameAgent);
  const qc = useQueryClient();

  const mutation = useMutation({
    mutationFn: async (name: string) => ipcAgentRename(agentId, name),
    onSuccess: (_r, name) => {
      renameAgent(agentId, name.trim());
      void qc.invalidateQueries({ queryKey: ['agents'] });
    },
    onError: (e) => setError(String(e)),
  });

  const dirty = draft.trim() !== currentName;
  const disabled = !dirty || mutation.isPending || draft.trim().length === 0;

  return (
    <div className="flex flex-col gap-1">
      <span className="text-11 uppercase tracking-wider text-text-tertiary">Name</span>
      <div className="flex items-center gap-2">
        <input
          type="text"
          value={draft}
          onChange={(e) => {
            setDraft(e.target.value);
            setError(null);
          }}
          className={cn(
            'flex-1 rounded-input border border-border bg-elevated px-3 py-2',
            'text-13 text-text-primary focus:border-accent focus:outline-none',
          )}
        />
        <button
          type="button"
          disabled={disabled}
          onClick={() => mutation.mutate(draft)}
          className={cn(
            'rounded-button bg-accent px-3 py-2 text-13 font-medium text-white',
            'disabled:opacity-40',
          )}
        >
          Save
        </button>
      </div>
      {error ? <span className="text-11 text-status-error">{error}</span> : null}
    </div>
  );
}

function TerminateButton({ agentId }: { agentId: string }): JSX.Element {
  const [confirming, setConfirming] = useState(false);
  const mutation = useMutation({
    mutationFn: () => ipcAgentTerminate(agentId),
  });
  return (
    <div className="mt-auto pt-6">
      {confirming ? (
        <div className="flex items-center justify-between rounded-panel border border-status-error/40 bg-status-error/10 p-3">
          <span className="text-13 text-status-error">Terminate this agent?</span>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setConfirming(false)}
              className="rounded-button px-3 py-1.5 text-13 text-text-secondary hover:text-text-primary"
            >
              Cancel
            </button>
            <button
              type="button"
              disabled={mutation.isPending}
              onClick={() => {
                mutation.mutate();
                setConfirming(false);
              }}
              className="rounded-button bg-status-error px-3 py-1.5 text-13 font-medium text-white hover:opacity-90"
            >
              Terminate
            </button>
          </div>
        </div>
      ) : (
        <button
          type="button"
          onClick={() => setConfirming(true)}
          className={cn(
            'w-full rounded-button border border-status-error/40 px-3 py-2',
            'text-13 text-status-error hover:bg-status-error/10',
          )}
        >
          Terminate agent
        </button>
      )}
    </div>
  );
}
