# ADR 0004 — Canvas state ownership: Zustand, not React Flow

- **Status:** Accepted
- **Date:** 2026-04-22

## Context

The Phase 2 canvas uses `@xyflow/react` (React Flow) as the rendering primitive for a spatial agent graph. React Flow ships with two usage patterns:

1. **Uncontrolled** — call `useNodesState` / `useEdgesState` inside the canvas component. React Flow owns every node's position, selection, drag state, and internal metadata.
2. **Controlled** — pass `nodes` / `edges` into `<ReactFlow>` as props, drive updates via the `onNodesChange` / `onEdgesChange` callbacks, and apply the changes yourself (usually via `applyNodeChanges`).

Orbit has several requirements that bleed past the canvas:

- Positions must be persisted to SQLite and rehydrated on app restart.
- The same agent is selectable from three surfaces (canvas node, sidebar DMs row, keyboard shortcut).
- The backend can push changes — e.g. a rename, a status transition, a future Phase 4 "message in flight" — that must update the canvas without user action.
- Per-agent chat state (draft, scroll, streaming) must survive selection changes.

## Decision

**Use the controlled pattern. The Zustand `useAgentsStore` is authoritative for every piece of canvas state we care about; React Flow owns only ephemeral view state (viewport transform, in-flight drag deltas).**

React Flow's `<ReactFlow>` receives a derived `nodes` array each render from Zustand; `onNodesChange` applies incoming changes back through `applyNodeChanges` and writes the result into `updateAgentPosition`. The drag-end handler is where positions get persisted to SQLite via `ipcAgentUpdatePosition`.

## Rationale

- **Single source of truth for positions.** The sidebar, the canvas, and the right panel all read the same `Agent.positionX/positionY`. If React Flow owned position internally, syncing it to the DB would require mirroring state — a classic source of drift bugs.
- **Backend push is natural.** When `agent:status_change` or a future broker event fires, we update the store and the canvas re-renders. No imperative calls into React Flow required.
- **Testable without a DOM.** The reducer logic that drives the canvas (positions, selection, streaming status) is plain Zustand — covered by Vitest without rendering React Flow.
- **Multi-surface selection is trivial.** `selectAgent(id)` in Zustand is enough; the canvas's `useEffect` watches `selectedAgentId` and smoothly pans to center.

## Tradeoffs

- **Every node re-creates on render.** The derived `nodes` array is a new object each time the store changes, and React Flow re-renders nodes unless we memoize the custom node component carefully. `AgentNode` uses `memo()` with a custom `propsAreEqual` that compares only the visual fields — this is load-bearing for 60fps with 10 agents on screen.
- **`onNodesChange` must be handled.** Forgetting to apply position deltas would make nodes undraggable. We apply them via `applyNodeChanges` and then propagate to the store inside the same callback.
- **Snap-to-grid needs care.** We snap on `onNodeDragStop` rather than every `change`, so drag feels continuous while the resting state is tidy.

## Alternatives considered

- **Uncontrolled React Flow + observer pattern.** Read positions back on drag end by dipping into React Flow's internal store. Rejected — couples us to React Flow internals and makes backend-push updates awkward.
- **Redux + React Flow sync middleware.** Too much ceremony for Phase 2; Zustand already has the agents map.
- **Skip React Flow, hand-roll canvas with pan/zoom/drag.** Months of work, much of it already solved well by `@xyflow/react`.

## Consequences

- All canvas-relevant fields live in `useAgentsStore` (`stores/agents.ts`). Add a field there, not inside React Flow.
- `AgentNode` must stay memoized. Any new visual prop that affects rendering needs to be added to `propsAreEqual`.
- Phase 4 message-flight animations will overlay the canvas with a separate component rather than adding React Flow edges; React Flow stays node-only.
