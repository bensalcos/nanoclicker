# Event-driven Windows shortcut

Review of global-hotkey 0.9.0 found that its Windows path starts a thread on each activation, polling GetAsyncKeyState every 50 ms until release. NanoClick no longer uses that listener; it retains the library as a HotKey parser.

The platform crate implements its own worker: thread-associated RegisterHotKey with MOD_NOREPEAT, a message queue and MsgWaitForMultipleObjects. Waiting handles messages and a manual shutdown event. OwnedHandle/Arc keep the event alive until the worker finishes; shutdown depends neither on polling nor on posting to a potentially recycled thread ID. FFI blocks document SAFETY preconditions.

Each registration has independent shared update state: a late callback from a replaced registration cannot invalidate the new one. A conflict preserves the previous registration.

Primary sources consulted:

- [RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey): thread queue with NULL hWnd and MOD_NOREPEAT.
- [MsgWaitForMultipleObjects](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjects): handles/messages waiting; do not close handles during a pending wait.
- [Virtual-key codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes): explicit letters, digits, F1–F24 and navigation mapping.

Validation at this change: two pure mapping tests passed on Linux, the then-current suite passed 24 tests, and Windows MSVC workspace cargo check/Clippy passed. The X11 GUI regression passed after refactoring shared dispatch. **Native Windows message-loop, conflict and held-key testing remains pending**; T11/T15 require that evidence. Later test totals and timer evidence are tracked separately in STATUS.md and native-waiter.md.
