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
        preferences: None,
        preferences_message: None,
        control,
        shortcut: Shortcut::unregistered(Desktop::X11, ctx, "F6".into()),
        desktop: Desktop::X11,
        probe,
        available: Some(Ok(())),
        theme: egui::ThemePreference::Dark,
    })
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
