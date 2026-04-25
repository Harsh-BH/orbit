# Development phases

Orbit is built in phases. Each phase ships a usable artifact — something a developer can actually run and use, not a scaffold. **Do not build ahead of the current phase.**

## Phase 0 — Foundation · _Complete_

Repo scaffold, three-panel shell, design tokens, CI.

## Phase 1 — One agent end-to-end · _Complete_

Spawn a single Claude Code subprocess, stream output to a chat panel, persist conversations in SQLite.

### Manual test checklist

1. **Spawn flow.** Click the `+` next to "Agents". Pick a name, emoji, color, and working directory. Click Spawn. The agent appears; the chat panel shows its header and an empty body.
2. **First message.** Type "list the files in this folder" in the chat input. Press Cmd/Ctrl+Enter. Assistant text streams in; a tool-call card appears with a ✓ when the tool returns.
3. **Tool call rendering.** Click the tool-call card; it expands to input + result. Click again to collapse.
4. **Restart persistence.** Close the app; reopen. The agent is still listed and the full conversation history is visible.
5. **Missing CLI.** Set `ORBIT_CLAUDE_PATH=/nope`; relaunch. You see the setup view with install instructions, not a crash.
6. **Termination.** Close the app while streaming. Confirm no orphan `claude` processes remain (`ps aux | grep claude`).
7. **Empty message.** Send button stays disabled; the command rejects empty input.
8. **Cross-platform.** Same flow on macOS, Windows, and Linux (requires `webkit2gtk-4.1`).

## Phase 2 — Canvas + multiple agents · _In Progress_

React Flow canvas with multiple agent nodes; each agent is an independent Claude Code subprocess with its own conversation, draft, and scroll state; selection syncs across canvas/sidebar/right panel.

**Deliverable:** launch the app, double-click an empty spot on the canvas to spawn agents, drag them around, switch between their chats, and close + reopen with positions and conversations intact.

### Manual test checklist

1. **Multi-spawn.** Double-click five different empty spots on the canvas; fill in the dialog each time. Five nodes appear at the clicked positions.
2. **Independent conversations.** Send "hello A" to the first agent. Switch to the second and send "hello B". Switch back — the first agent still shows its own history; no bleed.
3. **Drag + persist.** Drag each agent to a new position. Close the app, reopen — every agent is where you left it (snapped to 20px grid).
4. **Pan-to-selected.** Click an agent in the sidebar DMs list. Canvas smoothly pans to center on that agent.
5. **Status ring.** Send a long prompt to an agent, switch to another mid-stream. The first agent's node ring pulses green on the canvas even while you're looking at a different chat.
6. **Waiting-for-human heuristic.** Ask the agent a question the model is likely to bounce back with a question mark. Its node shows the amber `?` badge and the ring goes amber.
7. **Double-click → Settings.** Double-click a node. The right panel switches to Settings; Rename works end-to-end.
8. **Right-click menu.** Right-click a node. Focus chat, Rename, and Terminate all work. Terminate removes the subprocess (`ps aux | grep claude`).
9. **10-agent cap.** Try to spawn an 11th agent. The toast-style error message explains the limit.
10. **60fps pan/zoom.** Open DevTools > Performance; pan around aggressively with ten agents visible. Frame rate stays ≥ 50fps.
11. **Draft preservation.** Type a partial message in agent A's input, switch to agent B, switch back. The partial is still there.
12. **Scroll preservation.** Scroll halfway up in agent A's chat, switch to B, switch back. Scroll position is restored.
13. **Escape deselects.** Select an agent, press Escape. Selection clears.
14. **Cmd/Ctrl+Shift+N.** Opens the spawn dialog at the viewport center.
15. **Cross-platform.** Same flow on macOS, Windows, and Linux.

## Phase 3 — Agent identity + memory · _Planned_

Soul / Purpose / Memory fields on the agent. System prompt built by templating these three plus global context. Memory is persistent and editable.

## Phase 4 — Agent-to-agent messaging · _Planned_

Broker implementation, `send_message_to_agent` tool exposed to agents, canvas animations for in-flight messages. Messaging is audited in the DB.

## Phase 5 — Teams + folder access · _Planned_

Canvas team regions (visual groupings with bounds). Per-agent folder allowlist enforced at the core IPC boundary.

## Phase 6 — Git isolation · _Planned_

One git worktree per agent, one branch per agent. `git2`-based worktree manager. UI surfaces per-agent diff.

## Phase 7 — Tasks + status reports + sticky notes · _Planned_

Agents maintain a task list. Periodic status reports posted to a shared channel. Sticky notes placeable on the canvas for human annotation.

## Phase 8 — Group conversations + terminal + MCP · _Planned_

Group threads (multiple agents + human). xterm.js terminal tab for ad-hoc shell work. MCP server support so agents can use third-party tools.

## Beyond Phase 8

Out of scope until after Phase 8 ships: cloud sync, hardware sandboxing, roles system, manager agents, custom MCP authoring, mobile companion.
