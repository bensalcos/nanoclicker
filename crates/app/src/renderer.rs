//! Shared renderer selection for the application, QA target and diagnostics.

#[cfg(all(not(feature = "glow"), not(feature = "wgpu"), not(windows)))]
compile_error!("Enable a Linux renderer with the `wgpu` or `glow` feature.");

pub(crate) const fn selected() -> eframe::Renderer {
    // The Windows inbox display driver exposes only OpenGL 1.1. Prefer DirectX
    // through wgpu there; Linux defaults to the lower-memory measured GPU path.
    #[cfg(any(windows, all(not(feature = "glow"), feature = "wgpu")))]
    {
        eframe::Renderer::Wgpu
    }
    #[cfg(all(not(windows), feature = "glow"))]
    {
        eframe::Renderer::Glow
    }
}

pub(crate) const fn name() -> &'static str {
    #[cfg(all(not(windows), feature = "glow"))]
    {
        "glow"
    }
    #[cfg(any(windows, all(not(feature = "glow"), feature = "wgpu")))]
    {
        "wgpu"
    }
}
