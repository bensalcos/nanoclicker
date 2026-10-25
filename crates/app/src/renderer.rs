//! Shared renderer selection for the application, QA target and diagnostics.

pub(crate) const fn selected() -> eframe::Renderer {
    // The Windows inbox display driver exposes only OpenGL 1.1. Prefer DirectX
    // through wgpu there; keep the measured Linux default and build overrides.
    #[cfg(all(not(feature = "glow"), any(feature = "wgpu", windows)))]
    {
        eframe::Renderer::Wgpu
    }
    #[cfg(any(feature = "glow", all(not(feature = "wgpu"), not(windows))))]
    {
        eframe::Renderer::Glow
    }
}

pub(crate) const fn name() -> &'static str {
    match selected() {
        #[cfg(any(feature = "glow", not(windows)))]
        eframe::Renderer::Glow => "glow",
        #[cfg(any(feature = "wgpu", windows))]
        eframe::Renderer::Wgpu => "wgpu",
    }
}
