# UI bundle budget

The bundle gate totals `index.html` and every file in `ui/dist/assets`; deferred chunks count toward the same total. The committed baseline and 8% growth allowance remain unchanged when dependencies are upgraded.

The command palette uses a native modal dialog and combobox while retaining its six destinations, label filtering, and the exact fuzzy value scorer from published cmdk 1.1.1 (MIT license retained in the scorer). Browser verification covers keyboard selection, IME-safe Enter, modal focus and background inertness, scroll locking and restoration, focus restoration, Escape and backdrop dismissal. The default Vite browser targets remain unchanged; browser checks do not establish compatibility with every older Tauri WKWebView installation.
