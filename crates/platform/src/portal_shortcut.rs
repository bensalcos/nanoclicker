//! GlobalShortcuts sessions are configured by the Wayland desktop, not X11.
use ashpd::desktop::global_shortcuts::{GlobalShortcuts, NewShortcut};
use async_io::Timer;
use futures_lite::{
    StreamExt,
    future::{block_on, race},
};
use nanoclick_core::BackendError;
use std::{sync::Arc, thread::JoinHandle, time::Duration};

/// Authoritative portal-assigned shortcut or terminal failure.
#[derive(Clone, Debug)]
pub enum ShortcutUpdate {
    Registered(String),
    Unavailable(String),
}

/// A dedicated event-driven portal listener; Drop cancels permission requests.
pub struct PortalShortcut {
    stop: async_channel::Sender<()>,
    worker: Option<JoinHandle<()>>,
}
impl PortalShortcut {
    pub fn configure(
        update: impl Fn(ShortcutUpdate) + Send + Sync + 'static,
        activated: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, BackendError> {
        Self::configure_with_context(crate::wayland::PortalContext::default(), update, activated)
    }
    /// Binds a shortcut with the application's owned parent-window export.
    pub fn configure_with_context(
        context: crate::wayland::PortalContext,
        update: impl Fn(ShortcutUpdate) + Send + Sync + 'static,
        activated: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, BackendError> {
        let (stop, stopped) = async_channel::bounded(1);
        let update = Arc::new(update);
        let worker = std::thread::Builder::new()
            .name("nanoclick-portal-shortcut".into())
            .spawn(move || {
                let result = block_on(run(stopped, update.clone(), activated, context));
                if let Err(error) = result {
                    update(ShortcutUpdate::Unavailable(error.to_string()));
                }
            })
            .map_err(|e| BackendError::new(format!("Could not create shortcut listener: {e}")))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}
impl Drop for PortalShortcut {
    fn drop(&mut self) {
        let _ = self.stop.try_send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
async fn operation<T>(
    future: impl std::future::Future<Output = Result<T, ashpd::Error>>,
    stopped: &async_channel::Receiver<()>,
) -> Result<T, BackendError> {
    race(
        async {
            future
                .await
                .map_err(|e| BackendError::new(format!("Shortcut portal: {e}")))
        },
        race(
            async {
                let _ = stopped.recv().await;
                Err(BackendError::new("Shortcut configuration cancelled."))
            },
            async {
                Timer::after(Duration::from_secs(120)).await;
                Err(BackendError::new(
                    "Shortcut portal did not respond in time.",
                ))
            },
        ),
    )
    .await
}
async fn run(
    stopped: async_channel::Receiver<()>,
    update: Arc<impl Fn(ShortcutUpdate)>,
    activated: impl Fn(),
    context: crate::wayland::PortalContext,
) -> Result<(), BackendError> {
    let proxy = operation(
        async {
            GlobalShortcuts::with_connection(crate::portal_connection::connect().await?).await
        },
        &stopped,
    )
    .await?;
    let session = operation(proxy.create_session(Default::default()), &stopped).await?;
    let result = async {
        let mut pressed = operation(proxy.receive_activated(), &stopped).await?;
        let mut released = operation(proxy.receive_deactivated(), &stopped).await?;
        let mut closed = operation(session.receive_closed(), &stopped).await?;
        // Subscribe before binding so a restart cannot leave a registered-looking
        // shortcut attached to a session owned by the previous service.
        let mut owners = operation(
            async {
                proxy
                    .receive_owner_changed()
                    .await
                    .map_err(ashpd::Error::from)
            },
            &stopped,
        )
        .await?;
        let shortcuts =
            [NewShortcut::new("toggle", "Start or stop NanoClick").preferred_trigger("F6")];
        let bound = race(
            operation(
                proxy.bind_shortcuts(&session, &shortcuts, context.parent(), Default::default()),
                &stopped,
            ),
            async {
                race(
                    async {
                        let _ = closed.next().await;
                    },
                    async {
                        let _ = owners.next().await;
                    },
                )
                .await;
                Err(BackendError::new("The desktop shortcut session ended."))
            },
        )
        .await?
        .response()
        .map_err(|e| BackendError::new(e.to_string()))?;
        let trigger = bound
            .shortcuts()
            .iter()
            .find(|shortcut| shortcut.id() == "toggle")
            .ok_or_else(|| BackendError::new("The desktop did not assign a shortcut."))?
            .trigger_description()
            .to_owned();
        update(ShortcutUpdate::Registered(trigger));
        let mut held = false;
        loop {
            enum Signal {
                Press,
                Release,
                Closed,
                Stop,
            }
            let signal = race(
                async {
                    let _ = stopped.recv().await;
                    Signal::Stop
                },
                race(
                    async {
                        race(
                            async {
                                let _ = closed.next().await;
                            },
                            async {
                                let _ = owners.next().await;
                            },
                        )
                        .await;
                        Signal::Closed
                    },
                    race(
                        async {
                            match pressed.next().await {
                                Some(event) if event.shortcut_id() == "toggle" => Signal::Press,
                                _ => Signal::Closed,
                            }
                        },
                        async {
                            match released.next().await {
                                Some(event) if event.shortcut_id() == "toggle" => Signal::Release,
                                _ => Signal::Closed,
                            }
                        },
                    ),
                ),
            )
            .await;
            match signal {
                Signal::Stop => return Ok(()),
                Signal::Closed => {
                    return Err(BackendError::new("The desktop shortcut session ended."));
                }
                Signal::Release => held = false,
                Signal::Press if !held => {
                    held = true;
                    activated();
                }
                Signal::Press => {}
            }
        }
    }
    .await;
    // Cleanup is bounded separately even when the original request was cancelled.
    let _ = race(
        async {
            let _ = session.close().await;
        },
        async {
            Timer::after(Duration::from_secs(1)).await;
        },
    )
    .await;
    result
}
