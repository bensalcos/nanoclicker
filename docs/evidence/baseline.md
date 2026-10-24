# Parity and performance baseline

The functional reference is C++ commit 3fd5cd8. The Windows performance baseline is pending: this environment is Linux and the C++ binary was not executed.

Fixtures: intervals 1/10/100/999999 ms; Left/Right/Middle buttons; infinite mode and counts 1/10/999999; zero/out-of-range input; repeated Start; Stop during a long wait; active shutdown; partial down/up failure; occupied/held shortcut; automatic completion followed by restart. Large fixtures use fake time in core and bounded samples in native integration.

Intentional differences: English-only UI, explicit validation, manual Start with 3 s of cancelable preparation, skipped expired slots, checked delivery and failure recovery. Preserve an immediate first click from shortcuts and counts of complete pairs.

Do not claim improvement over C++ without a comparable baseline. Measure the engine, UI and target separately under docs/VALIDATION.md.
