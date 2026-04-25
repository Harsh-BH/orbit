import { beforeEach, describe, expect, it } from 'vitest';
import type { Agent, AgentEvent } from '@orbit/types';
import { useAgentsStore, deriveStatus } from './agents';

function makeAgent(id: string): Agent {
  return {
    id,
    name: 'A',
    emoji: '🌟',
    color: '#5E6AD2',
    workingDir: '/tmp',
    sessionId: null,
    modelOverride: null,
    status: 'idle',
    soul: null,
    purpose: null,
    memory: null,
    folderAccess: '[]',
    teamId: null,
    positionX: 0,
    positionY: 0,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  };
}

function reset(): void {
  useAgentsStore.setState({
    agents: {},
    orderedAgentIds: [],
    selectedAgentId: null,
    messagesByAgent: {},
    streamingByAgent: {},
    lastErrorByAgent: {},
    chatDraftByAgent: {},
    chatScrollByAgent: {},
    draggingAgentId: null,
  });
}

describe('agents store / applyEvent', () => {
  beforeEach(reset);

  it('session_started writes the session id back onto the agent', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    useAgentsStore.getState().applyEvent('a', {
      type: 'session_started',
      session_id: 'sess-42',
    });
    expect(useAgentsStore.getState().agents['a']?.sessionId).toBe('sess-42');
  });

  it('accumulates text_delta events into the streaming buffer', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    const deltas: AgentEvent[] = [
      { type: 'text_delta', content: 'Hel' },
      { type: 'text_delta', content: 'lo, ' },
      { type: 'text_delta', content: 'world' },
    ];
    for (const e of deltas) useAgentsStore.getState().applyEvent('a', e);
    expect(useAgentsStore.getState().streamingByAgent['a']?.text).toBe('Hello, world');
  });

  it('tool_use_start adds an in-flight tool call', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    useAgentsStore.getState().applyEvent('a', {
      type: 'tool_use_start',
      tool_id: 't1',
      tool_name: 'Read',
      input: { path: 'a.ts' },
    });
    const calls = useAgentsStore.getState().streamingByAgent['a']?.toolCalls ?? [];
    expect(calls).toHaveLength(1);
    expect(calls[0]?.toolId).toBe('t1');
    expect(calls[0]?.complete).toBe(false);
    expect(calls[0]?.result).toBeNull();
  });

  it('tool_use_complete flips the matching call to complete', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    useAgentsStore.getState().applyEvent('a', {
      type: 'tool_use_start',
      tool_id: 't1',
      tool_name: 'Read',
      input: {},
    });
    useAgentsStore.getState().applyEvent('a', {
      type: 'tool_use_complete',
      tool_id: 't1',
      tool_name: 'Read',
      input: { path: 'x.ts' },
    });
    const c = useAgentsStore.getState().streamingByAgent['a']?.toolCalls[0];
    expect(c?.complete).toBe(true);
    expect((c?.input as { path: string }).path).toBe('x.ts');
  });

  it('tool_use_result fills in result + isError on the matching call', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    useAgentsStore.getState().applyEvent('a', {
      type: 'tool_use_start',
      tool_id: 't1',
      tool_name: 'Bash',
      input: { command: 'ls' },
    });
    useAgentsStore.getState().applyEvent('a', {
      type: 'tool_use_result',
      tool_id: 't1',
      result: 'a\nb\nc',
      is_error: false,
    });
    const c = useAgentsStore.getState().streamingByAgent['a']?.toolCalls[0];
    expect(c?.result).toBe('a\nb\nc');
    expect(c?.isError).toBe(false);
  });

  it('turn_complete clears the streaming buffer', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    useAgentsStore.getState().applyEvent('a', { type: 'text_delta', content: 'hi' });
    useAgentsStore.getState().applyEvent('a', {
      type: 'turn_complete',
      usage: {
        inputTokens: 10,
        outputTokens: 5,
        cacheReadTokens: 0,
        cacheCreationTokens: 0,
      },
    });
    expect(useAgentsStore.getState().streamingByAgent['a']).toBeNull();
  });

  it('error event populates lastErrorByAgent and clears streaming', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    useAgentsStore.getState().applyEvent('a', { type: 'text_delta', content: 'partial' });
    useAgentsStore.getState().applyEvent('a', {
      type: 'error',
      message: 'model refused',
      recoverable: true,
    });
    expect(useAgentsStore.getState().lastErrorByAgent['a']).toBe('model refused');
    expect(useAgentsStore.getState().streamingByAgent['a']).toBeNull();
  });

  it('thinking_delta is currently a no-op (Phase 3 will wire it)', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    useAgentsStore.getState().applyEvent('a', { type: 'thinking_delta', content: 'ponder' });
    expect(useAgentsStore.getState().streamingByAgent['a']).toBeUndefined();
  });

  it('hydrate replaces the agent set and picks the first as active', () => {
    useAgentsStore.getState().hydrate([makeAgent('a'), makeAgent('b')]);
    expect(useAgentsStore.getState().selectedAgentId).toBe('a');
    expect(useAgentsStore.getState().orderedAgentIds).toEqual(['a', 'b']);
  });

  it('removeAgent drops the row, its messages, its streaming state, and rewires selectedAgentId', () => {
    useAgentsStore.getState().hydrate([makeAgent('a'), makeAgent('b')]);
    useAgentsStore.getState().applyEvent('a', { type: 'text_delta', content: 'x' });
    useAgentsStore.getState().setMessages('a', [
      {
        id: 'm',
        conversationId: 'c',
        role: 'user',
        content: '{}',
        createdAt: new Date().toISOString(),
      },
    ]);

    useAgentsStore.getState().removeAgent('a');
    expect(useAgentsStore.getState().agents['a']).toBeUndefined();
    expect(useAgentsStore.getState().selectedAgentId).toBe('b');
    expect(useAgentsStore.getState().messagesByAgent['a']).toBeUndefined();
    expect(useAgentsStore.getState().streamingByAgent['a']).toBeUndefined();
  });

  it('appendPersistedMessage is idempotent on duplicate ids', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    const m = {
      id: 'dup',
      conversationId: 'c',
      role: 'user',
      content: '{}',
      createdAt: new Date().toISOString(),
    };
    useAgentsStore.getState().appendPersistedMessage('a', m);
    useAgentsStore.getState().appendPersistedMessage('a', m);
    expect(useAgentsStore.getState().messagesByAgent['a']?.length).toBe(1);
  });
});

describe('agents store / Phase 2', () => {
  beforeEach(reset);

  it('selectAgent replaces the selected id and tolerates null', () => {
    useAgentsStore.getState().hydrate([makeAgent('a'), makeAgent('b')]);
    useAgentsStore.getState().selectAgent('b');
    expect(useAgentsStore.getState().selectedAgentId).toBe('b');
    useAgentsStore.getState().selectAgent(null);
    expect(useAgentsStore.getState().selectedAgentId).toBeNull();
  });

  it('updateAgentPosition writes both axes onto the agent', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    useAgentsStore.getState().updateAgentPosition('a', { x: 120, y: -48 });
    const a = useAgentsStore.getState().agents['a'];
    expect(a?.positionX).toBe(120);
    expect(a?.positionY).toBe(-48);
  });

  it('per-agent chat drafts are preserved across agent switches', () => {
    useAgentsStore.getState().hydrate([makeAgent('a'), makeAgent('b')]);
    useAgentsStore.getState().setChatDraft('a', 'hello from A');
    useAgentsStore.getState().setChatDraft('b', 'hello from B');
    useAgentsStore.getState().selectAgent('b');
    expect(useAgentsStore.getState().chatDraftByAgent['a']).toBe('hello from A');
    expect(useAgentsStore.getState().chatDraftByAgent['b']).toBe('hello from B');
  });

  it('per-agent scroll offsets are preserved across agent switches', () => {
    useAgentsStore.getState().hydrate([makeAgent('a'), makeAgent('b')]);
    useAgentsStore.getState().setChatScroll('a', 480);
    useAgentsStore.getState().setChatScroll('b', 120);
    expect(useAgentsStore.getState().chatScrollByAgent['a']).toBe(480);
    expect(useAgentsStore.getState().chatScrollByAgent['b']).toBe(120);
  });

  it('streaming states do not bleed across agents', () => {
    useAgentsStore.getState().hydrate([makeAgent('a'), makeAgent('b')]);
    const events: AgentEvent[] = [{ type: 'text_delta', content: 'A says hi' }];
    for (const e of events) useAgentsStore.getState().applyEvent('a', e);
    expect(useAgentsStore.getState().streamingByAgent['a']?.text).toBe('A says hi');
    expect(useAgentsStore.getState().streamingByAgent['b']).toBeUndefined();
  });

  it('renameAgent updates the name field', () => {
    useAgentsStore.getState().upsertAgent(makeAgent('a'));
    useAgentsStore.getState().renameAgent('a', 'New Name');
    expect(useAgentsStore.getState().agents['a']?.name).toBe('New Name');
  });

  it('removeAgent also clears the agent draft and scroll entries', () => {
    useAgentsStore.getState().hydrate([makeAgent('a')]);
    useAgentsStore.getState().setChatDraft('a', 'draft');
    useAgentsStore.getState().setChatScroll('a', 500);
    useAgentsStore.getState().removeAgent('a');
    expect(useAgentsStore.getState().chatDraftByAgent['a']).toBeUndefined();
    expect(useAgentsStore.getState().chatScrollByAgent['a']).toBeUndefined();
  });
});

describe('deriveStatus', () => {
  const a = makeAgent('a');

  it('returns "active" when a stream is in progress', () => {
    expect(deriveStatus(a, { text: 'hi', toolCalls: [], usage: null }, null, null)).toBe('active');
  });

  it('returns "error" when the agent.status is error or lastError is set', () => {
    expect(deriveStatus({ ...a, status: 'error' }, null, null, null)).toBe('error');
    expect(deriveStatus(a, null, 'boom', null)).toBe('error');
  });

  it('returns "waiting_for_human" when the last assistant text ends with ?', () => {
    expect(deriveStatus(a, null, null, 'Can you confirm?')).toBe('waiting_for_human');
  });

  it('returns "idle" otherwise', () => {
    expect(deriveStatus(a, null, null, 'ok')).toBe('idle');
    expect(deriveStatus(a, null, null, null)).toBe('idle');
  });
});
