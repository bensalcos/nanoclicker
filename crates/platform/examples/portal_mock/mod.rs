//! Minimal RemoteDesktop and GlobalShortcuts wire fixture; never connects to a compositor.
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};
use zbus::{
    Connection, fdo,
    message::Header,
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Str},
};

pub const NAME: &str = "org.freedesktop.portal.Desktop";
type Options = HashMap<String, OwnedValue>;
#[derive(Clone, Copy)]
pub enum Mode {
    Allow,
    Legacy,
    DenyRegistration,
    Deny,
    CloseDuringSelect,
    HoldStart,
    FailRelease,
    HoldPress,
    HoldRelease,
}
#[derive(Default)]
pub struct State {
    pub identities: HashMap<String, String>,
    pub registrations: usize,
    pub session: Option<OwnedObjectPath>,
    pub buttons: Vec<(i32, u32)>,
    pub closes: usize,
    pub starts: usize,
    pub persist_modes: Vec<u32>,
    pub restore_tokens: Vec<Option<String>>,
    release_failed: bool,
}
pub type Shared = Arc<Mutex<State>>;
pub fn lock(state: &Shared) -> MutexGuard<'_, State> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
fn error(value: impl std::fmt::Display) -> fdo::Error {
    fdo::Error::Failed(value.to_string())
}
fn path(
    header: &Header<'_>,
    options: &Options,
    prefix: &str,
    token: &str,
) -> fdo::Result<OwnedObjectPath> {
    let sender = header
        .sender()
        .ok_or_else(|| error("Missing sender"))?
        .as_str()
        .trim_start_matches(':')
        .replace('.', "_");
    let token = options
        .get(token)
        .and_then(|value| <&str>::try_from(value).ok())
        .ok_or_else(|| error("Missing handle token"))?;
    OwnedObjectPath::try_from(format!(
        "/org/freedesktop/portal/desktop/{prefix}/{sender}/{token}"
    ))
    .map_err(error)
}
async fn response(
    connection: &Connection,
    header: &Header<'_>,
    options: &Options,
    code: u32,
    results: Options,
) -> fdo::Result<OwnedObjectPath> {
    let request = path(header, options, "request", "handle_token")?;
    connection
        .emit_signal(
            None::<&str>,
            request.as_str(),
            "org.freedesktop.portal.Request",
            "Response",
            &(code, results),
        )
        .await
        .map_err(error)?;
    Ok(request)
}
pub async fn closed(connection: &Connection, session: &OwnedObjectPath) -> zbus::Result<()> {
    connection
        .emit_signal(
            None::<&str>,
            session.as_str(),
            "org.freedesktop.portal.Session",
            "Closed",
            &Options::new(),
        )
        .await
}
struct FakeSession(Shared);
#[zbus::interface(name = "org.freedesktop.portal.Session")]
impl FakeSession {
    fn close(&self) {
        lock(&self.0).closes += 1;
    }
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }
}
pub struct Portal {
    pub state: Shared,
    pub mode: Mode,
}
#[zbus::interface(name = "org.freedesktop.portal.RemoteDesktop")]
impl Portal {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        2
    }
    #[zbus(property)]
    fn available_device_types(&self) -> u32 {
        2
    }
    async fn create_session(
        &self,
        options: Options,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> fdo::Result<OwnedObjectPath> {
        let sender = header.sender().ok_or_else(|| error("Missing sender"))?;
        if !matches!(self.mode, Mode::Legacy)
            && lock(&self.state)
                .identities
                .get(sender.as_str())
                .map(String::as_str)
                != Some("io.github.bensalcos.NanoClick")
        {
            return Err(fdo::Error::AccessDenied("An app id is required".into()));
        }
        let session = path(&header, &options, "session", "session_handle_token")?;
        connection
            .object_server()
            .at(session.as_str(), FakeSession(self.state.clone()))
            .await
            .map_err(error)?;
        lock(&self.state).session = Some(session.clone());
        let results = HashMap::from([(
            "session_handle".into(),
            OwnedValue::from(Str::from(session.as_str())),
        )]);
        response(connection, &header, &options, 0, results).await
    }
    async fn select_devices(
        &self,
        session: ObjectPath<'_>,
        options: Options,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> fdo::Result<OwnedObjectPath> {
        {
            let mut state = lock(&self.state);
            state.persist_modes.push(
                options
                    .get("persist_mode")
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(0),
            );
            state.restore_tokens.push(
                options
                    .get("restore_token")
                    .and_then(|value| <&str>::try_from(value).ok())
                    .map(str::to_owned),
            );
        }
        if matches!(self.mode, Mode::CloseDuringSelect) {
            closed(connection, &session.into()).await.map_err(error)?;
        }
        response(
            connection,
            &header,
            &options,
            u32::from(matches!(self.mode, Mode::Deny)),
            Options::new(),
        )
        .await
    }
    async fn start(
        &self,
        _session: ObjectPath<'_>,
        _parent: &str,
        options: Options,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> fdo::Result<OwnedObjectPath> {
        lock(&self.state).starts += 1;
        if matches!(self.mode, Mode::HoldStart) {
            return path(&header, &options, "request", "handle_token");
        }
        response(
            connection,
            &header,
            &options,
            0,
            HashMap::from([
                ("devices".into(), OwnedValue::from(2u32)),
                (
                    "restore_token".into(),
                    OwnedValue::from(Str::from(format!(
                        "mock-token-{}",
                        lock(&self.state).starts
                    ))),
                ),
            ]),
        )
        .await
    }
    async fn notify_pointer_button(
        &self,
        _session: ObjectPath<'_>,
        _options: Options,
        button: i32,
        state: u32,
    ) -> fdo::Result<()> {
        {
            let mut observed = lock(&self.state);
            if observed.buttons.len() >= 8 {
                return Err(error("Unexpected extra input"));
            }
            observed.buttons.push((button, state));
            if matches!(self.mode, Mode::FailRelease) && state == 0 && !observed.release_failed {
                observed.release_failed = true;
                return Err(error("Injected release failure"));
            }
        }
        if matches!(
            (self.mode, state),
            (Mode::HoldPress, 1) | (Mode::HoldRelease, 0)
        ) {
            async_io::Timer::after(std::time::Duration::from_secs(5)).await;
        }
        Ok(())
    }
}

struct Shortcuts(Shared, Mode);
#[zbus::interface(name = "org.freedesktop.portal.GlobalShortcuts")]
impl Shortcuts {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }
    async fn create_session(
        &self,
        options: Options,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> fdo::Result<OwnedObjectPath> {
        Portal {
            state: self.0.clone(),
            mode: self.1,
        }
        .create_session(options, header, connection)
        .await
    }
    async fn bind_shortcuts(
        &self,
        session: ObjectPath<'_>,
        _shortcuts: Vec<(String, Options)>,
        _parent: &str,
        options: Options,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> fdo::Result<OwnedObjectPath> {
        lock(&self.0).starts += 1;
        if matches!(self.1, Mode::CloseDuringSelect) {
            closed(connection, &session.into()).await.map_err(error)?;
        }
        if matches!(self.1, Mode::CloseDuringSelect | Mode::HoldStart) {
            return path(&header, &options, "request", "handle_token");
        }
        let shortcuts = vec![(
            "toggle",
            HashMap::from([
                (
                    "description",
                    OwnedValue::from(Str::from("Start or stop NanoClick")),
                ),
                ("trigger_description", OwnedValue::from(Str::from("F6"))),
            ]),
        )];
        let value = OwnedValue::try_from(zbus::zvariant::Value::new(shortcuts)).map_err(error)?;
        response(
            connection,
            &header,
            &options,
            0,
            HashMap::from([("shortcuts".into(), value)]),
        )
        .await
    }
}

struct Registry(Shared, Mode);
#[zbus::interface(name = "org.freedesktop.host.portal.Registry")]
impl Registry {
    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }
    fn register(
        &self,
        app_id: &str,
        _options: Options,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<()> {
        if matches!(self.1, Mode::DenyRegistration) {
            return Err(fdo::Error::AccessDenied("Host registration denied".into()));
        }
        let sender = header.sender().ok_or_else(|| error("Missing sender"))?;
        let mut state = lock(&self.0);
        if state.identities.contains_key(sender.as_str()) {
            return Err(fdo::Error::Failed("Peer already registered".into()));
        }
        state.identities.insert(sender.to_string(), app_id.into());
        state.registrations += 1;
        Ok(())
    }
}

pub async fn serve(mode: Mode) -> zbus::Result<(Connection, Shared)> {
    let state = Shared::default();
    let connection = Connection::session().await?;
    if !matches!(mode, Mode::Legacy) {
        connection
            .object_server()
            .at(
                "/org/freedesktop/portal/desktop",
                Registry(state.clone(), mode),
            )
            .await?;
    }
    connection
        .object_server()
        .at(
            "/org/freedesktop/portal/desktop",
            Portal {
                state: state.clone(),
                mode,
            },
        )
        .await?;
    connection
        .object_server()
        .at(
            "/org/freedesktop/portal/desktop",
            Shortcuts(state.clone(), mode),
        )
        .await?;
    // Do not replace an existing service, even if the isolation guard is misused.
    connection
        .request_name_with_flags(
            NAME,
            fdo::RequestNameFlags::AllowReplacement | fdo::RequestNameFlags::DoNotQueue,
        )
        .await?;
    Ok((connection, state))
}
