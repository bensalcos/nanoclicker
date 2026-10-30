use super::*;
use nanoclick_core::{BackendError, InputSink};

struct NoInput;
impl InputSink for NoInput {
    fn click(&mut self, _: MouseButton) -> Result<(), BackendError> {
        Err(BackendError::new("Test fixture never sends desktop input"))
    }
}
fn fixture(ctx: &egui::Context) -> Result<NanoClick, Box<dyn std::error::Error>> {
    let engine = Engine::new(|_| Ok(NoInput), || {})?;
    let (_, probe) = mpsc::channel();
    let mut control = Controller::new(engine);
    control.draft.finite = true;
    Ok(NanoClick {
        diagnostics: None,
        diagnostics_message: None,
        preferences: None,
        preferences_message: None,
        saved_preferences: Preferences::from_draft(
            &control.draft,
            egui::ThemePreference::Dark,
            "F6",
        )?,
        confirm_close: false,
        close_approved: false,
        preferences_recovery_available: false,
        control,
        shortcut: Shortcut::unregistered(Desktop::X11, ctx, "F6".into()),
        desktop: Desktop::X11,
        probe,
        available: Some(Ok(())),
        theme: egui::ThemePreference::Dark,
        #[cfg(target_os = "windows")]
        window_fit: None,
    })
}

#[test]
fn shortcuts_received_while_unavailable_are_not_replayed() -> Result<(), Box<dyn std::error::Error>>
{
    let ctx = egui::Context::default();
    for availability in [None, Some(Err("Probe failed".into()))] {
        let mut app = fixture(&ctx)?;
        app.available = availability;
        app.shortcut.activation()();
        app.process_shortcuts(&ctx);
        assert!(!app.shortcut.has_pending());
        app.available = Some(Ok(()));
        app.process_shortcuts(&ctx);
        assert_eq!(app.control.engine.snapshot().state, RunState::Idle);
    }
    let mut app = fixture(&ctx)?;
    app.available = None;
    app.shortcut.activation()();
    app.complete_probe(Ok(()));
    app.process_shortcuts(&ctx);
    assert_eq!(app.control.engine.snapshot().state, RunState::Idle);
    Ok(())
}

#[test]
fn shortcut_restart_waits_for_stop_acknowledgement() -> Result<(), Box<dyn std::error::Error>> {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        thread,
        time::{Duration, Instant},
    };
    struct Release(mpsc::SyncSender<()>);
    impl Drop for Release {
        fn drop(&mut self) {
            let _ = self.0.try_send(());
        }
    }
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx)?;
    let (entered, ready) = mpsc::sync_channel(1);
    let (permit, blocked) = mpsc::sync_channel(1);
    let first = AtomicBool::new(true);
    app.control = Controller::new(Engine::new(
        move |_| {
            if first.swap(false, Ordering::AcqRel) {
                entered
                    .send(())
                    .map_err(|error| BackendError::new(error.to_string()))?;
                blocked
                    .recv()
                    .map_err(|error| BackendError::new(error.to_string()))?;
            }
            Ok(NoInput)
        },
        || {},
    )?);
    // Release before the engine is dropped, including assertion/error unwinding.
    let release = Release(permit);
    app.control.toggle(false);
    ready.recv_timeout(Duration::from_secs(1))?;
    app.control.stop();
    app.shortcut.activation()();
    app.process_shortcuts(&ctx);
    let retained = app.shortcut.has_pending();
    release.0.send(())?;
    let deadline = Instant::now() + Duration::from_secs(1);
    while app.control.engine.snapshot().state.is_active() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(1));
    }
    assert!(
        retained,
        "Restart must remain queued while cleanup is blocked"
    );
    assert!(!app.control.engine.snapshot().state.is_active());
    app.process_shortcuts(&ctx);
    assert_eq!(app.control.engine.snapshot().run_id, 2);
    assert!(!app.shortcut.has_pending());
    Ok(())
}

#[test]
fn explicit_stop_discards_pending_shortcut_starts() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx)?;
    app.shortcut.activation()();
    app.stop();
    app.process_shortcuts(&ctx);
    assert_eq!(app.control.engine.snapshot().run_id, 0);
    assert!(!app.shortcut.has_pending());
    Ok(())
}
fn frame(app: &mut NanoClick, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(520.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
    output
}
fn key(key: egui::Key, shift: bool) -> Vec<egui::Event> {
    let modifiers = egui::Modifiers {
        shift,
        ..Default::default()
    };
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers,
        })
        .collect()
}
fn node_name(
    id: egui::accesskit::NodeId,
    nodes: &[(egui::accesskit::NodeId, egui::accesskit::Node)],
) -> String {
    let Some((_, node)) = nodes.iter().find(|(candidate, _)| *candidate == id) else {
        return String::new();
    };
    if let Some(name) = node.label().or_else(|| {
        matches!(node.role(), egui::Role::Label | egui::Role::TextRun)
            .then(|| node.value())
            .flatten()
    }) {
        return name.to_owned();
    }
    node.labelled_by()
        .iter()
        .map(|id| node_name(*id, nodes))
        .collect::<Vec<_>>()
        .join(" ")
}
fn focused_name(output: &egui::FullOutput) -> Result<String, Box<dyn std::error::Error>> {
    let tree = output
        .platform_output
        .accesskit_update
        .as_ref()
        .ok_or("Missing accessibility tree")?;
    Ok(node_name(tree.focus, &tree.nodes))
}
#[test]
fn keyboard_follows_reading_order_in_both_directions() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = egui::Context::default();
    theme::configure(&ctx);
    ctx.enable_accesskit();
    let mut app = fixture(&ctx)?;
    frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
    for expected in [
        "Theme",
        "Save",
        "Interval",
        "Mouse button",
        "Until stopped",
        "Count",
        "Count",
        "Global shortcut",
        "Apply",
        "Start",
    ] {
        let output = frame(&mut app, &ctx, key(egui::Key::Tab, false));
        assert_eq!(focused_name(&output)?, expected);
    }
    for expected in ["Apply", "Global shortcut", "Count"] {
        frame(&mut app, &ctx, key(egui::Key::Tab, true)).drop_without_applying_deltas();
        // egui deliberately applies reverse traversal on the following frame.
        let output = frame(&mut app, &ctx, vec![]);
        assert_eq!(focused_name(&output)?, expected);
    }
    Ok(())
}

#[test]
fn enter_in_an_edit_field_never_starts_the_engine() -> Result<(), Box<dyn std::error::Error>> {
    for target in ["Interval", "Global shortcut"] {
        let ctx = egui::Context::default();
        theme::configure(&ctx);
        ctx.enable_accesskit();
        let mut app = fixture(&ctx)?;
        frame(&mut app, &ctx, vec![]).drop_without_applying_deltas();
        let mut reached = false;
        for _ in 0..10 {
            let output = frame(&mut app, &ctx, key(egui::Key::Tab, false));
            if focused_name(&output)? == target {
                reached = true;
                break;
            }
        }
        assert!(reached, "Field must be reachable by keyboard: {target}");
        frame(&mut app, &ctx, key(egui::Key::Enter, false)).drop_without_applying_deltas();
        assert_eq!(app.control.engine.snapshot().state, RunState::Idle);
    }
    Ok(())
}

#[test]
fn primary_action_stays_inside_minimum_window_with_long_status_and_errors()
-> Result<(), Box<dyn std::error::Error>> {
    for (state, expected) in [
        (RunState::Idle, "Start"),
        (RunState::Preparing, "Cancel"),
        (RunState::Running, "Stop"),
        (RunState::Stopping, "Stopping…"),
        (RunState::Faulted, "Start"),
    ] {
        let ctx = egui::Context::default();
        theme::configure(&ctx);
        ctx.enable_accesskit();
        let mut app = fixture(&ctx)?;
        app.control.error = Some("An intentionally long error message must scroll with the form rather than displacing Stop. ".repeat(12));
        let mut snapshot = Snapshot {
            state,
            ..Default::default()
        };
        snapshot.metrics.sent = u64::MAX;
        snapshot.metrics.skipped = u64::MAX;
        if state == RunState::Preparing {
            snapshot.ready_at = Some(std::time::Instant::now() + Duration::from_secs(3));
        }
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(420.0, 520.0),
                )),
                ..Default::default()
            },
            |ui| app.show_snapshot(ui, &snapshot),
        );
        output.textures_delta.clear();
        let tree = output
            .platform_output
            .accesskit_update
            .as_ref()
            .ok_or("Missing accessibility tree")?;
        let status = tree
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.role() == egui::Role::Status)
            .ok_or("Session status lacks a live region")?;
        assert_eq!(status.live(), Some(egui::accesskit::Live::Polite));
        assert!(status.is_live_atomic());
        let counter = tree
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| {
                node.value()
                    .is_some_and(|value| value.contains("pairs sent"))
            })
            .ok_or("Progress counter is absent")?;
        assert_eq!(
            counter.live().unwrap_or(egui::accesskit::Live::Off),
            egui::accesskit::Live::Off
        );
        let status_text = status.value().ok_or("Visible status lacks text")?;
        assert_eq!(
            status.label(),
            Some(status_text),
            "Live status needs a name, not only a string value, for native announcements"
        );
        let action = tree
            .nodes
            .iter()
            .map(|(_, node)| node)
            .find(|node| node.role() == egui::Role::Button && node.label() == Some(expected))
            .ok_or("Primary action is absent")?;
        let rect = action.bounds().ok_or("Primary action lacks bounds")?;
        assert!(
            rect.x0 >= 0.0 && rect.y0 >= 0.0 && rect.x1 <= 420.0 && rect.y1 <= 520.0,
            "{state:?}: {rect:?}"
        );
        assert!(
            rect.height() >= 46.0,
            "Primary action must retain its target size"
        );
    }
    Ok(())
}

#[test]
fn close_requires_a_decision_and_failed_save_keeps_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx)?;
    assert!(!app.preferences_dirty());
    app.control.draft.count = "500".into();
    assert!(app.preferences_dirty());
    app.request_close(&ctx);
    assert!(app.confirm_close);
    app.shortcut.activation()();
    app.process_shortcuts(&ctx);
    assert!(!app.shortcut.has_pending());
    app.resolve_close(&ctx, CloseChoice::Save);
    assert!(
        app.confirm_close,
        "A failed Save must not close or discard edits"
    );
    app.resolve_close(&ctx, CloseChoice::Cancel);
    assert!(!app.confirm_close);
    assert_eq!(app.control.draft.count, "500");
    app.request_close(&ctx);
    app.resolve_close(&ctx, CloseChoice::Discard);
    assert!(app.close_approved);
    Ok(())
}
#[test]
fn successful_save_clears_dirty_state() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx)?;
    let dir = tempfile::tempdir()?;
    let mut file = PreferencesFile::new(dir.path().join("preferences.json"));
    file.load()?;
    app.preferences = Some(file);
    app.control.draft.count = "500".into();
    assert!(app.save_preferences());
    assert!(!app.preferences_dirty());
    app.control.draft.count = "invalid".into();
    assert!(app.preferences_dirty());
    assert!(!app.save_preferences());
    Ok(())
}

#[test]
fn remembering_portal_assignment_does_not_save_other_unsaved_edits()
-> Result<(), Box<dyn std::error::Error>> {
    let ctx = egui::Context::default();
    let mut app = fixture(&ctx)?;
    let dir = tempfile::tempdir()?;
    let mut file = PreferencesFile::new(dir.path().join("preferences.json"));
    file.load()?;
    app.preferences = Some(file);
    app.control.draft.interval = "37".into();
    app.shortcut.portal_configured = true;
    app.remember_portal_assignment();
    let stored = app
        .preferences
        .as_mut()
        .ok_or("Missing fixture file")?
        .load()?;
    assert!(stored.portal_shortcut_configured);
    assert_eq!(stored.interval_ms, 100);
    assert!(app.preferences_dirty());
    Ok(())
}

#[test]
fn discard_uses_event_modifiers_even_after_alt_is_released()
-> Result<(), Box<dyn std::error::Error>> {
    let ctx = egui::Context::default();
    theme::configure(&ctx);
    let mut app = fixture(&ctx)?;
    app.control.draft.count = "500".into();
    app.request_close(&ctx);
    frame(
        &mut app,
        &ctx,
        vec![egui::Event::Key {
            key: egui::Key::D,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::ALT,
        }],
    )
    .drop_without_applying_deltas();
    assert!(app.close_approved);
    Ok(())
}
