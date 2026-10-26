//! Windows monitor work area in physical virtual-desktop coordinates.
use std::num::NonZeroIsize;
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
};

/// Queries the current window's nearest monitor, including its taskbar exclusion.
pub fn work_area(window: NonZeroIsize) -> Option<[i32; 4]> {
    // SAFETY: Windows validates the opaque window handle. The output points to
    // initialized storage with the size required by GetMonitorInfoW.
    unsafe {
        let monitor = MonitorFromWindow(window.get() as _, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if monitor.is_null() || GetMonitorInfoW(monitor, &mut info) == 0 {
            return None;
        }
        let work = info.rcWork;
        Some([work.left, work.top, work.right, work.bottom])
    }
}
