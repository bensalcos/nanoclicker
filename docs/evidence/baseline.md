# Parity and performance baseline

The functional reference is C++ commit 3fd5cd8. A Windows 11 VM is now available. The published v1.0 asset was downloaded and its GitHub digest verified (`cc2573d1ac31d1dc474dd8dc992e688660df4b10f1a1487e7f087297a930f7e4`), but PE imports include MSVCP140D.dll, VCRUNTIME140D.dll, VCRUNTIME140_1D.dll and ucrtbased.dll. It is not yet an executed release-performance baseline. A comparable optimized C++ build and measurements remain pending; no Rust improvement over C++ is claimed.

Fixtures: intervals 1/10/100/999999 ms; Left/Right/Middle buttons; infinite mode and counts 1/10/999999; zero/out-of-range input; repeated Start; Stop during a long wait; active shutdown; partial down/up failure; occupied/held shortcut; automatic completion followed by restart. Large fixtures use fake time in core and bounded samples in native integration.

Intentional differences: English-only UI, explicit validation, manual Start with 3 s of cancelable preparation, skipped expired slots, checked delivery and failure recovery. Preserve an immediate first click from shortcuts and counts of complete pairs.

Do not claim improvement over C++ without a comparable baseline. Measure the engine, UI and target separately under docs/VALIDATION.md.
