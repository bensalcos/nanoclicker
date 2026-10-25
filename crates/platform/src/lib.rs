//! Native desktop adapters for NanoClick.
use nanoclick_core::{BackendError, Cancellation, InputSink, MouseButton};

#[cfg(target_os = "linux")]
pub mod portal_shortcut;
#[cfg(target_os = "linux")]
pub mod wayland;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(any(target_os = "windows", test))]
mod windows_key;
#[cfg(target_os = "windows")]
pub mod windows_shortcut;
#[cfg(any(target_os = "windows", test))]
mod windows_time;
#[cfg(target_os = "windows")]
mod windows_waiter;
#[cfg(target_os = "linux")]
mod x11;
#[cfg(target_os = "linux")]
pub mod x11_shortcut;
#[cfg(target_os = "linux")]
mod x11_stream;

/// Selected transport; capability probes still verify the actual connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Desktop {
    Windows,
    X11,
    Wayland,
}
impl Desktop {
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::Windows
        }
        #[cfg(not(target_os = "windows"))]
        {
            if std::env::var_os("WAYLAND_DISPLAY").is_some()
                || std::env::var("XDG_SESSION_TYPE").is_ok_and(|s| s == "wayland")
            {
                Self::Wayland
            } else {
                Self::X11
            }
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::X11 => "Linux · X11",
            Self::Wayland => "Linux · Wayland",
        }
    }
    /// Read-only connection/capability test. Never opens a permission dialog.
    pub fn probe(self) -> Result<(), BackendError> {
        match self {
            #[cfg(target_os = "windows")]
            Self::Windows => Ok(()),
            #[cfg(target_os = "linux")]
            Self::X11 => x11::X11Input::open(&Cancellation::default()).map(|_| ()),
            #[cfg(target_os = "linux")]
            Self::Wayland => wayland::probe(),
            #[allow(unreachable_patterns)]
            _ => Err(BackendError::new(
                "This desktop backend is unavailable in this build.",
            )),
        }
    }
}

/// Platform-specific resources allocated once during run preparation.
pub enum NativeInput {
    #[cfg(target_os = "windows")]
    Windows(windows::WindowsInput),
    #[cfg(target_os = "linux")]
    X11(Box<x11::X11Input>),
    #[cfg(target_os = "linux")]
    Wayland(Box<wayland::WaylandInput>),
}
impl NativeInput {
    pub fn open(desktop: Desktop, cancel: &Cancellation) -> Result<Self, BackendError> {
        if cancel.is_cancelled() {
            return Err(BackendError::new("Preparation cancelled."));
        }
        match desktop {
            #[cfg(target_os = "windows")]
            Desktop::Windows => Ok(Self::Windows(windows::WindowsInput::open())),
            #[cfg(target_os = "linux")]
            Desktop::X11 => x11::X11Input::open(cancel).map(Box::new).map(Self::X11),
            #[cfg(target_os = "linux")]
            Desktop::Wayland => wayland::WaylandInput::open(cancel)
                .map(Box::new)
                .map(Self::Wayland),
            #[allow(unreachable_patterns)]
            _ => Err(BackendError::new(
                "This desktop backend is unavailable in this build.",
            )),
        }
    }
}
impl InputSink for NativeInput {
    fn click(&mut self, button: MouseButton) -> Result<(), BackendError> {
        match self {
            #[cfg(target_os = "windows")]
            Self::Windows(sink) => sink.click(button),
            #[cfg(target_os = "linux")]
            Self::X11(sink) => sink.click(button),
            #[cfg(target_os = "linux")]
            Self::Wayland(sink) => sink.click(button),
        }
    }
    fn finish(&mut self) -> Result<(), BackendError> {
        match self {
            #[cfg(target_os = "windows")]
            Self::Windows(sink) => sink.finish(),
            #[cfg(target_os = "linux")]
            Self::X11(sink) => sink.finish(),
            #[cfg(target_os = "linux")]
            Self::Wayland(sink) => sink.finish(),
        }
    }
}

/// Selects the worker's monotonic wait without changing the input transport.
pub fn waiter() -> Result<impl nanoclick_core::Waiter, BackendError> {
    #[cfg(target_os = "windows")]
    {
        windows_waiter::WindowsWaiter::new()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(nanoclick_core::StdWaiter::default())
    }
}
