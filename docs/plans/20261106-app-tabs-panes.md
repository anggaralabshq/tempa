---
task_id: 20261106-app-tabs-panes
state: approved
approved_by: wisnuanggara
approved_at: 2026-11-06
implementer:
reviewer:
---
## Tujuan

M2 task 4/7: tabs and split panes, each pane one terminal. Splitting or closing a pane never restarts another pane's shell; closing a pane or tab kills its shells; a shell that exits closes its pane.

## Langkah

1. panes.ts (pure): split, close, leaves, rects.
2. tabs.ts (pure reducer): new, select, closeTab, split, focus, closePane.
3. shortcuts.ts: Cmd+T/D/Shift+D/W on macOS; Ctrl+Shift+T/E/O/W elsewhere (plain Ctrl+D/W stay with the shell).
4. Workspace.tsx renders all panes flat, keyed by id, positioned from rects; focused pane gets keyboard focus.

## File yang disentuh

app/src/layout/{panes.ts,tabs.ts,shortcuts.ts,Workspace.tsx} + tests, app/src/terminal/Terminal.tsx, app/src/App.tsx.

## Risiko

Hidden tabs keep their shells running (intended); panes are equal halves (no drag-resize yet). GUI behavior is checked by the owner in the running app, not by the agent.
