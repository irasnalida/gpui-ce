use crate::{DevicePixels, Size};
use std::sync::Arc;
use windows_061::Win32::Graphics::Direct3D11::ID3D11Texture2D;

/// A Windows Graphics Capture frame backed by its native D3D11 texture.
#[derive(Clone)]
pub struct WindowsScreenCaptureFrame {
    texture: Arc<ID3D11Texture2D>,
    size: Size<DevicePixels>,
    display_time: u64,
}

impl WindowsScreenCaptureFrame {
    #[cfg(feature = "screen-capture")]
    pub(crate) fn new(
        texture: ID3D11Texture2D,
        size: Size<DevicePixels>,
        display_time: u64,
    ) -> Self {
        Self {
            texture: Arc::new(texture),
            size,
            display_time,
        }
    }

    /// Wraps an arbitrary D3D11 texture as a surface frame, for example a decoded video frame.
    ///
    /// The texture must be a `DXGI_FORMAT_B8G8R8A8_UNORM` (or RGBA) 2D texture with the
    /// `D3D11_BIND_SHADER_RESOURCE` flag, created on the same `ID3D11Device` that the window's
    /// renderer uses. Passing a texture that another `Arc` still references lets the producer
    /// check `Arc::strong_count` to learn when the renderer has released the frame.
    ///
    /// `presentation_time` is stored as the frame's display time (100-nanosecond units).
    pub fn from_d3d11_texture(
        texture: Arc<ID3D11Texture2D>,
        size: Size<DevicePixels>,
        presentation_time: u64,
    ) -> Self {
        Self {
            texture,
            size,
            display_time: presentation_time,
        }
    }

    /// Returns the native texture containing this frame.
    pub fn texture(&self) -> &ID3D11Texture2D {
        &self.texture
    }

    /// Returns the frame dimensions in device pixels.
    pub fn size(&self) -> Size<DevicePixels> {
        self.size
    }

    /// Returns the capture timestamp in 100-nanosecond units.
    pub fn display_time(&self) -> u64 {
        self.display_time
    }
}

impl std::fmt::Debug for WindowsScreenCaptureFrame {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WindowsScreenCaptureFrame")
            .field("size", &self.size)
            .field("display_time", &self.display_time)
            .finish_non_exhaustive()
    }
}
