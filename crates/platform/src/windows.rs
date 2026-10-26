use nanoclick_core::{BackendError, InputSink, MouseButton};
#[cfg(test)]
use std::{cell::RefCell, collections::VecDeque};
#[cfg(not(test))]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::SendInput;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
};

pub struct WindowsInput {
    pairs: [[INPUT; 2]; 3],
    pressed: Option<usize>,
    #[cfg(test)]
    send_results: RefCell<VecDeque<u32>>,
    #[cfg(test)]
    send_calls: RefCell<Vec<usize>>,
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
            #[cfg(test)]
            send_results: RefCell::default(),
            #[cfg(test)]
            send_calls: RefCell::default(),
        }
    }
    #[inline]
    fn send(&self, inputs: &[INPUT]) -> u32 {
        #[cfg(test)]
        {
            self.send_calls.borrow_mut().push(inputs.len());
            return self
                .send_results
                .borrow_mut()
                .pop_front()
                .unwrap_or_default();
        }
        #[cfg(not(test))]
        {
            // SAFETY: `inputs` points to initialized INPUT records and remains alive
            // throughout SendInput; Windows does not retain this pointer.
            unsafe {
                SendInput(
                    inputs.len() as u32,
                    inputs.as_ptr(),
                    size_of::<INPUT>() as i32,
                )
            }
        }
    }
    fn release(&mut self) -> Result<(), BackendError> {
        if let Some(index) = self.pressed {
            let inserted = self.send(std::slice::from_ref(&self.pairs[index][1]));
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
        let inserted = self.send(&self.pairs[index]);
        // SendInput inserts the INPUT array in order: only a single accepted event
        // leaves the button down and requires a matching release during cleanup.
        self.pressed = (inserted == 1).then_some(index);
        if inserted == 2 {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn input_with_results(results: &[u32]) -> WindowsInput {
        let input = WindowsInput::open();
        input
            .send_results
            .borrow_mut()
            .extend(results.iter().copied());
        input
    }

    #[test]
    fn zero_inserted_events_do_not_attempt_an_unmatched_release() {
        let mut input = input_with_results(&[0]);

        assert!(input.click(MouseButton::Left).is_err());
        assert!(input.pressed.is_none());
        assert!(input.finish().is_ok());
        assert_eq!(*input.send_calls.borrow(), [2]);
    }

    #[test]
    fn one_inserted_down_event_is_released_during_cleanup() {
        let mut input = input_with_results(&[1, 1]);

        assert!(input.click(MouseButton::Left).is_err());
        assert_eq!(input.pressed, Some(0));
        assert!(input.finish().is_ok());
        assert!(input.pressed.is_none());
        assert_eq!(*input.send_calls.borrow(), [2, 1]);
    }

    #[test]
    fn failed_release_remains_tracked_for_a_later_cleanup_attempt() {
        let mut input = input_with_results(&[1, 0, 1]);

        assert!(input.click(MouseButton::Left).is_err());
        assert!(input.finish().is_err());
        assert_eq!(input.pressed, Some(0));
        assert!(input.finish().is_ok());
        assert!(input.pressed.is_none());
        assert_eq!(*input.send_calls.borrow(), [2, 1, 1]);
    }
}
