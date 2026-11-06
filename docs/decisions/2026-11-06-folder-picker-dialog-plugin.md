## Konteks

Adding a project needs a native folder picker on macOS and Windows.

## Opsi

1. tauri-plugin-dialog (official Tauri plugin) 2.8.4, Rust and JS.
2. A text field for the path: error-prone, no native picker.
3. Drag-and-drop a folder onto the window only: not discoverable.

## Keputusan

Option 1, pinned to 2.8.4 on both sides; capability limited to dialog:allow-open.

## Konsekuensi

One more official plugin to keep in step with Tauri releases. No file-system access is granted to the web view: the backend canonicalizes and validates the chosen path.

## Agent + model

claude-code / claude-opus-5-5
