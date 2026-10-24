use nanoclick_core::{BackendError, InputSink, MouseButton};
use x11rb::{
    connection::Connection, protocol::xtest::ConnectionExt, rust_connection::RustConnection,
};

/// XTEST connection owned by the worker, never shared with the GUI event loop.
pub struct X11Input {
    connection: RustConnection,
    pressed: Option<u8>,
}
fn error(error: impl std::fmt::Display) -> BackendError {
    BackendError::new(format!("X11: {error}"))
}
impl X11Input {
    pub fn open() -> Result<Self, BackendError> {
        let (connection, _) = x11rb::connect(None).map_err(error)?;
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
            self.connection
                .xtest_fake_input(5, button, 0, x11rb::NONE, 0, 0, 0)
                .map_err(error)?
                .check()
                .map_err(error)?;
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
        self.pressed = Some(button);
        let down = self
            .connection
            .xtest_fake_input(4, button, 0, x11rb::NONE, 0, 0, 0)
            .map_err(error)?;
        let up = self
            .connection
            .xtest_fake_input(5, button, 0, x11rb::NONE, 0, 0, 0)
            .map_err(error)?;
        self.connection.flush().map_err(error)?;
        down.check().map_err(error)?;
        up.check().map_err(error)?;
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
