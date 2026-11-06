## Konteks

The desktop app must react to file changes in each project (spec 6.4): sync after source edits, alert on direct edits to generated files. It needs native file events on macOS (FSEvents), Windows (ReadDirectoryChangesW) and Linux (inotify).

## Opsi

1. `notify` v8 (recommended_watcher, recursive). 2. Polling the tree on a timer: no dependency, but slow to react and costly on big repos. 3. notify-debouncer-full: an extra crate for what a small loop does. 4. A Tauri fs-watch plugin in the frontend: puts contract logic in the UI, against the spec (actions call tempa-core).

## Keputusan

Option 1, in app/src-tauri only. Debounce is a small burst collector over std mpsc.

## Konsekuensi

One more dependency in the app (not in tempa-core or the CLI). Linux inotify watch limits apply to very large repos; revisit in M4.

## Agent + model

claude-code / claude-opus-5-5
