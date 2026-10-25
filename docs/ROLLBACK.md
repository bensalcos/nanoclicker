# Use the C++ reference or return to Rust

The immutable reference is commit `3fd5cd8863b316437e3e50339a1dce8b84aad215`. The migration branch preserves its C++ sources. Preview packaging performs no installation or autostart registration.

To inspect or build the original revision without resetting the Rust checkout, create a separate directory that does not already exist:

```sh
git worktree add --detach ../nanoclick-cpp-reference 3fd5cd8863b316437e3e50339a1dce8b84aad215
git -C ../nanoclick-cpp-reference rev-parse HEAD
```

On Windows, the same Git command can use `..\nanoclick-cpp-reference` as its directory. The original revision contains `Autoclicker.slnx` and `Autoclicker.vcxproj`. The project declares C++20, Windows SDK 10.0, toolset v145 and a Release/x64 configuration. Open that solution with a compatible installed Visual Studio toolchain and build Release/x64. This Windows build was not executed in the Linux migration environment; the original source is the reference, not a newly certified fallback binary.

Before switching applications, Stop the active session and close its window. Start the chosen application explicitly. The original application does not import the Rust preference schema; leave the Rust files in place rather than deleting or translating them.

To return to Rust, run the preview executable or use `cargo run --locked -p nanoclick` in the migration checkout. Rust starts idle and never resumes a saved running session or permission grant. This procedure changes neither branch history nor the user's existing files, and it does not require replacing either checkout.

The C++ source will be removed only in a separate change after the [acceptance matrix](ACCEPTANCE.md) closes. Git history preserves the reference after any future retirement.
