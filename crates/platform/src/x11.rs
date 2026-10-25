use crate::x11_stream::{self, X11Connection};
use nanoclick_core::{BackendError, Cancellation, InputSink, MouseButton};
use x11rb::protocol::xtest::ConnectionExt;

/// XTEST connection owned by the worker, never shared with the GUI event loop.
pub struct X11Input {
    connection: X11Connection,
    pressed: Option<u8>,
}
fn error(error: impl std::fmt::Display) -> BackendError {
    BackendError::new(format!("X11: {error}"))
}
fn unverified(error: impl std::fmt::Display) -> BackendError {
    BackendError::new(format!(
        "X11: {error}. Input delivery or release could not be verified."
    ))
}
impl X11Input {
    pub fn open(cancel: &Cancellation) -> Result<Self, BackendError> {
        let (connection, _) = x11_stream::connect(cancel).map_err(error)?;
        connection
            .xtest_get_version(2, 2)
            .map_err(error)?
            .reply()
            .map_err(error)?;
        Ok(Self {
            connection,
            pressed: None,
        })
    }
    fn release(&mut self) -> Result<(), BackendError> {
        if let Some(button) = self.pressed {
            self.connection.stream().begin_request();
            self.connection
                .xtest_fake_input(5, button, 0, x11rb::NONE, 0, 0, 0)
                .map_err(unverified)?
                .check()
                .map_err(unverified)?;
            self.pressed = None;
        }
        Ok(())
    }
}
impl InputSink for X11Input {
    fn click(&mut self, button: MouseButton) -> Result<(), BackendError> {
        let button = match button {
            MouseButton::Left => 1,
            MouseButton::Middle => 2,
            MouseButton::Right => 3,
        };
        self.connection.stream().begin_request();
        self.pressed = Some(button);
        let down = self
            .connection
            .xtest_fake_input(4, button, 0, x11rb::NONE, 0, 0, 0)
            .map_err(unverified)?;
        let up = self
            .connection
            .xtest_fake_input(5, button, 0, x11rb::NONE, 0, 0, 0)
            .map_err(unverified)?;
        // Checking the cookies flushes both requests together with the server
        // synchronization. A preceding flush would split that into two writes.
        down.check().map_err(unverified)?;
        up.check().map_err(unverified)?;
        self.pressed = None;
        Ok(())
    }
    fn finish(&mut self) -> Result<(), BackendError> {
        self.release()
    }
}
impl Drop for X11Input {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
