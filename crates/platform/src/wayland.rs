//! Consent-based input through the XDG RemoteDesktop portal.
//! API reference: https://docs.rs/ashpd/0.13.13/ashpd/desktop/remote_desktop/
use ashpd::desktop::{
    Session,
    remote_desktop::{DeviceType, KeyState, RemoteDesktop, SelectDevicesOptions},
};
use async_io::Timer;
use futures_lite::{
    StreamExt,
    future::{block_on, race},
};
use nanoclick_core::{BackendError, Cancellation, InputSink, MouseButton};
use std::{future::Future, sync::Arc, thread::JoinHandle, time::Duration};

fn error(error: impl std::fmt::Display) -> BackendError {
    BackendError::new(format!("Wayland portal: {error}"))
}

fn bounded<T>(
    future: impl Future<Output = Result<T, ashpd::Error>>,
    timeout: Duration,
    cancel: Option<&Cancellation>,
) -> Result<T, BackendError> {
    block_on(race(async { future.await.map_err(error) }, async {
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if cancel.is_some_and(Cancellation::is_cancelled) {
                return Err(BackendError::new("Preparation cancelled."));
            }
            if std::time::Instant::now() >= deadline {
                return Err(BackendError::new(
                    "The desktop portal did not respond in time.",
                ));
            }
            Timer::after(Duration::from_millis(10)).await;
        }
    }))
}

/// Read-only capability probe; it does not request permission.
pub fn probe() -> Result<(), BackendError> {
    bounded(
        async {
            let proxy = RemoteDesktop::new().await?;
            let devices = proxy.available_device_types().await?;
            if devices.contains(DeviceType::Pointer) {
                Ok(())
            } else {
                Err(ashpd::Error::NoResponse)
            }
        },
        Duration::from_secs(3),
        None,
    )
}

pub struct WaylandInput {
    proxy: RemoteDesktop,
    session: Arc<Session<RemoteDesktop>>,
    pressed: Option<i32>,
    closed: bool,
    stop_monitor: async_channel::Sender<()>,
    monitor: Option<JoinHandle<()>>,
}
impl WaylandInput {
    pub fn open(cancel: &Cancellation) -> Result<Self, BackendError> {
        let proxy = bounded(RemoteDesktop::new(), Duration::from_secs(3), Some(cancel))?;
        let session = Arc::new(bounded(
            proxy.create_session(Default::default()),
            Duration::from_secs(3),
            Some(cancel),
        )?);
        let result = bounded(
            async {
                proxy
                    .select_devices(
                        &session,
                        SelectDevicesOptions::default()
                            .set_devices(Some(DeviceType::Pointer.into())),
                    )
                    .await?
                    .response()?;
                let response = proxy
                    .start(&session, None, Default::default())
                    .await?
                    .response()?;
                if !response.devices().contains(DeviceType::Pointer) {
                    return Err(ashpd::Error::NoResponse);
                }
                Ok(())
            },
            Duration::from_secs(120),
            Some(cancel),
        );
        if let Err(error) = result {
            let _ = bounded(session.close(), Duration::from_secs(1), None);
            return Err(error);
        }
        let (stop_monitor, stopped) = async_channel::bounded(1);
        let monitored = session.clone();
        let token = cancel.clone();
        let (ready, subscribed) = std::sync::mpsc::sync_channel(1);
        let monitor = std::thread::Builder::new()
            .name("nanoclick-portal-session".into())
            .spawn(move || {
                block_on(race(
                    async {
                        let mut closed = match monitored.receive_closed().await {
                            Ok(stream) => {
                                let _ = ready.send(Ok(()));
                                stream
                            }
                            Err(error) => {
                                let _ = ready.send(Err(error.to_string()));
                                return;
                            }
                        };
                        let _ = closed.next().await;
                        if !token.is_cancelled() {
                            token.fail(BackendError::new(
                                "Desktop permission was revoked or the portal disconnected.",
                            ));
                        }
                    },
                    async {
                        let _ = stopped.recv().await;
                    },
                ));
            })
            .map_err(error)?;
        match subscribed.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => Ok(Self {
                proxy,
                session,
                pressed: None,
                closed: false,
                stop_monitor,
                monitor: Some(monitor),
            }),
            result => {
                let _ = stop_monitor.try_send(());
                let _ = monitor.join();
                let _ = bounded(session.close(), Duration::from_secs(1), None);
                Err(BackendError::new(format!(
                    "Could not monitor desktop permission: {result:?}"
                )))
            }
        }
    }
    fn release(&mut self) -> Result<(), BackendError> {
        if let Some(button) = self.pressed {
            bounded(
                self.proxy.notify_pointer_button(
                    &self.session,
                    button,
                    KeyState::Released,
                    Default::default(),
                ),
                Duration::from_secs(1),
                None,
            )?;
            self.pressed = None;
        }
        Ok(())
    }
}
impl InputSink for WaylandInput {
    fn click(&mut self, button: MouseButton) -> Result<(), BackendError> {
        let button = match button {
            MouseButton::Left => 0x110,
            MouseButton::Right => 0x111,
            MouseButton::Middle => 0x112,
        };
        self.pressed = Some(button);
        bounded(
            self.proxy.notify_pointer_button(
                &self.session,
                button,
                KeyState::Pressed,
                Default::default(),
            ),
            Duration::from_secs(1),
            None,
        )?;
        self.release()
    }
    fn finish(&mut self) -> Result<(), BackendError> {
        if self.closed {
            return Ok(());
        }
        let _ = self.stop_monitor.try_send(());
        if let Some(monitor) = self.monitor.take() {
            let _ = monitor.join();
        }
        let released = self.release();
        let closed = bounded(self.session.close(), Duration::from_secs(1), None);
        self.closed = true;
        released.and(closed)
    }
}
impl Drop for WaylandInput {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}
