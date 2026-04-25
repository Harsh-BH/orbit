import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/react';
import type { NodeProps } from '@xyflow/react';
import { AgentNode, type AgentNodeData } from './agent-node';

function mkProps(overrides: Partial<AgentNodeData> = {}): NodeProps {
  const data: AgentNodeData = {
    agentId: 'a',
    name: 'Scout',
    emoji: '🛰️',
    color: '#5E6AD2',
    currentTask: '',
    status: 'idle',
    selected: false,
    ...overrides,
  };
  return {
    id: 'a',
    type: 'agentNode',
    data,
    selected: false,
    dragging: false,
    deletable: true,
    draggable: true,
    selectable: true,
    isConnectable: false,
    zIndex: 0,
    positionAbsoluteX: 0,
    positionAbsoluteY: 0,
  };
}

describe('AgentNode', () => {
  it('renders the emoji, name, and task line', () => {
    const { getByText, container } = render(
      <AgentNode {...mkProps({ currentTask: 'listing files' })} />,
    );
    expect(getByText('Scout')).toBeInTheDocument();
    expect(getByText('listing files')).toBeInTheDocument();
    expect(container.textContent).toContain('🛰️');
  });

  it('shows the help badge only when waiting_for_human', () => {
    const idle = render(<AgentNode {...mkProps({ status: 'idle' })} />);
    expect(idle.queryByLabelText('Waiting for a human response')).toBeNull();
    idle.unmount();

    const waiting = render(<AgentNode {...mkProps({ status: 'waiting_for_human' })} />);
    expect(waiting.queryByLabelText('Waiting for a human response')).not.toBeNull();
  });

  it('applies the pulsing class only when active', () => {
    const active = render(<AgentNode {...mkProps({ status: 'active' })} />);
    const activeEls = active.container.querySelectorAll('.orbit-pulse');
    expect(activeEls.length).toBeGreaterThan(0);
    active.unmount();

    const idle = render(<AgentNode {...mkProps({ status: 'idle' })} />);
    expect(idle.container.querySelectorAll('.orbit-pulse').length).toBe(0);
  });

  it('sets the data-agent-id attribute so testability tools can target nodes', () => {
    const { container } = render(<AgentNode {...mkProps()} />);
    expect(container.querySelector('[data-agent-id="a"]')).not.toBeNull();
  });

  it('tolerates a long task string by truncating visually (no overflow)', () => {
    const longTask = 'x'.repeat(500);
    const { getByText } = render(<AgentNode {...mkProps({ currentTask: longTask })} />);
    const el = getByText(longTask);
    // The element is present; the .truncate class is what keeps it on one line.
    expect(el.className).toContain('truncate');
  });
});
