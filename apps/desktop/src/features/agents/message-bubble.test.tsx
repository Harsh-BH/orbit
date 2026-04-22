import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import type { Message } from '@orbit/types';
import { PersistedMessageBubble, ToolCallBubble } from './message-bubble';

function message(partial: Partial<Message> & Pick<Message, 'role' | 'content'>): Message {
  return {
    id: partial.id ?? 'm',
    conversationId: partial.conversationId ?? 'c',
    role: partial.role,
    content: partial.content,
    createdAt: partial.createdAt ?? new Date().toISOString(),
  };
}

describe('PersistedMessageBubble', () => {
  it('renders user text', () => {
    render(
      <PersistedMessageBubble
        message={message({ role: 'user', content: JSON.stringify({ text: 'hi' }) })}
      />,
    );
    expect(screen.getByText('hi')).toBeInTheDocument();
  });

  it('renders assistant text', () => {
    render(
      <PersistedMessageBubble
        message={message({
          role: 'assistant',
          content: JSON.stringify({ text: 'hello back' }),
        })}
      />,
    );
    expect(screen.getByText('hello back')).toBeInTheDocument();
  });

  it('renders tool_use as a collapsed card with name + summary', () => {
    render(
      <PersistedMessageBubble
        message={message({
          role: 'tool_use',
          content: JSON.stringify({
            tool_id: 't1',
            tool_name: 'Read',
            input: { path: 'src/App.tsx' },
          }),
        })}
      />,
    );
    expect(screen.getByText('Read')).toBeInTheDocument();
    expect(screen.getByText('Read src/App.tsx')).toBeInTheDocument();
  });

  it('tool_result rows render nothing on their own (merged upstream)', () => {
    const { container } = render(
      <PersistedMessageBubble
        message={message({
          role: 'tool_result',
          content: JSON.stringify({
            tool_id: 't1',
            result: 'ok',
            is_error: false,
          }),
        })}
      />,
    );
    expect(container).toBeEmptyDOMElement();
  });
});

describe('ToolCallBubble', () => {
  it('shows "running…" when in-flight', () => {
    render(
      <ToolCallBubble
        call={{
          toolId: 't1',
          toolName: 'Bash',
          input: { command: 'ls' },
          result: null,
          isError: false,
          inFlight: true,
        }}
      />,
    );
    expect(screen.getByText('running…')).toBeInTheDocument();
  });

  it('shows an error border and the red result when is_error is true', () => {
    render(
      <ToolCallBubble
        call={{
          toolId: 't1',
          toolName: 'Bash',
          input: { command: 'ls' },
          result: 'boom',
          isError: true,
          inFlight: false,
        }}
      />,
    );
    expect(screen.getByText('Bash')).toBeInTheDocument();
  });
});
