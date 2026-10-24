use nanoclick_core::{BackendError, InputSink, MouseButton};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT, SendInput,
};

pub struct WindowsInput {
    pairs: [[INPUT; 2]; 3],
    pressed: Option<usize>,
}
fn event(flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
impl WindowsInput {
    pub fn open() -> Self {
        Self {
            pairs: [
                [event(MOUSEEVENTF_LEFTDOWN), event(MOUSEEVENTF_LEFTUP)],
                [event(MOUSEEVENTF_RIGHTDOWN), event(MOUSEEVENTF_RIGHTUP)],
                [event(MOUSEEVENTF_MIDDLEDOWN), event(MOUSEEVENTF_MIDDLEUP)],
            ],
            pressed: None,
        }
    }
    fn release(&mut self) -> Result<(), BackendError> {
        if let Some(index) = self.pressed {
            // SAFETY: pointer references one initialized INPUT, and cbSize matches its ABI.
            let inserted =
                unsafe { SendInput(1, &self.pairs[index][1], size_of::<INPUT>() as i32) };
            if inserted != 1 {
                return Err(BackendError::new(
                    "Windows could not release the mouse button.",
                ));
            }
            self.pressed = None;
        }
        Ok(())
    }
}
impl InputSink for WindowsInput {
    fn click(&mut self, button: MouseButton) -> Result<(), BackendError> {
        let index = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
        };
        self.pressed = Some(index);
        // SAFETY: the array contains two initialized INPUT values and remains alive
        // throughout SendInput; Windows does not retain this pointer.
        let inserted =
            unsafe { SendInput(2, self.pairs[index].as_ptr(), size_of::<INPUT>() as i32) };
        if inserted == 2 {
            self.pressed = None;
            return Ok(());
        }
        Err(BackendError::new(format!(
            "Windows accepted {inserted} of 2 mouse events. Input to elevated applications may be blocked."
        )))
    }
    fn finish(&mut self) -> Result<(), BackendError> {
        self.release()
    }
}
impl Drop for WindowsInput {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
