# X11 shortcut mapping regressions

Date: 2026-10-24. Production fix: `afbf033`. Reproduce with `scripts/verify-x11.sh`, which always creates a private Xvfb. No physical keyboard or pointer mapping is changed.

The previous listener treated every core MappingNotify as a keyboard change. A new test swapped two entries in the private server's pointer-button map, restored the original map, then pressed F6. It failed with “Pointer mapping invalidated keyboard shortcut: Keyboard mapping changed. Apply the shortcut again.” Pointer-only notifications now leave the keyboard binding active.

The accompanying keyboard regression changed F6's keysyms to F7 and restored the original symbols. Before explicit XKB subscriptions, it timed out without the required failure notification. The listener now subscribes to XKB map/new-keyboard events before reading its binding snapshot. Genuine map events still invalidate the binding and tell the user to reapply it. An initial blanket invalidation of new-keyboard events broke the existing held-key test: XTEST can switch the master device without changing its map. New-device notifications therefore compare keycode bounds, symbols and modifiers against the registered snapshot; the binding survives only when those are unchanged.

The final native fixture passes all three buttons, outside-focus activation, held-key suppression, conflict preservation, pointer-map preservation and explicit keyboard-map invalidation. The application release build, all 42 local Rust tests and X11-feature Clippy checks pass. A twenty-cycle release-UI regression also passes in Xvfb. The listener remains event driven; metadata queries occur during registration/device changes, not on every click.

This does not accept every modifier/group/layout combination or key-held-during-registration case. Native Windows and real Wayland shortcut acceptance remain separate. Earlier failed attempts are retained locally under artifacts/; they are not accepted runs.
