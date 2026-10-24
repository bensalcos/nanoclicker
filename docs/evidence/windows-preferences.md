# Blocked preference directories on Windows

The first hosted Windows run at `5b55daf` reproduced a real loader bug: [`failed_publication_preserves_existing_path`](https://github.com/bensalcos/nanoclicker/actions/runs/40088616210/job/120856520993) failed because `load()` incorrectly succeeded when an existing parent component was a regular file.

Fixture: create a file named `blocked`, then load `blocked/preferences.json`. Unix reports a non-directory error. Windows normalizes this path failure to `NotFound`; the loader previously interpreted every `NotFound` as a first launch, returned defaults and enabled replacement. Save still failed at directory creation, but the unusable configuration location was not rejected at load time as intended.

The missing-file branch now checks ancestors until it reaches an existing directory. A non-directory or other filesystem error remains a load error, with replacement protected. Genuinely absent nested directories still allow defaults and a first atomic save.

The original failing test is retained and strengthened to require `SettingsError::Protected` on Save. A new test covers creating missing nested directories, guarding against an overbroad fix that rejects normal first launches. Seven settings tests and the full 39-test workspace pass locally; Linux/Windows target Clippy pass. Hosted run 40089494911 at 2ecaf23 passed both regression cases and all 41 Windows tests, followed by the MSVC release build. The blocked-parent bug is therefore verified fixed on the Windows Server 2025 runner; Windows 11 GUI acceptance remains separate.
