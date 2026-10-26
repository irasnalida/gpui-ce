//! Access to the Direct3D 11 device that GPUI's renderer draws with.
//!
//! Video decoders, capture pipelines and other producers of GPU textures need to create their
//! resources on the same device as the renderer. Textures from a different device cannot be
//! sampled by the renderer without shared handles and explicit synchronization.

use std::sync::{
    RwLock,
    atomic::{AtomicU64, Ordering},
};

use windows::{
    Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11Multithread},
    core::Interface as _,
};
use windows_061::Win32::Graphics::Direct3D11::ID3D11Device as ID3D11Device061;

static SHARED_DEVICE: RwLock<Option<ID3D11Device>> = RwLock::new(None);
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Records the device the renderer is currently using. Called whenever devices are created or
/// recreated after a device-lost event.
pub(crate) fn publish_device(device: &ID3D11Device) {
    // Producers on other threads use the device's immediate context (decoding, video
    // processing, copies). The immediate context is not thread-safe unless this is enabled.
    if let Ok(multithread) = device.cast::<ID3D11Multithread>() {
        unsafe {
            let _ = multithread.SetMultithreadProtected(true);
        }
    } else {
        log::warn!("ID3D11Multithread unavailable; shared device is not thread-safe");
    }
    if let Ok(mut slot) = SHARED_DEVICE.write() {
        *slot = Some(device.clone());
        GENERATION.fetch_add(1, Ordering::SeqCst);
    }
}

/// Returns the `ID3D11Device` used by GPUI's DirectX renderer, or `None` before the platform has
/// created its devices (or when running headless).
///
/// The device is returned as a `windows` 0.61 interface, matching the texture type of
/// `gpui::WindowsScreenCaptureFrame`. Callers using another `windows` version can move the COM
/// pointer across with `Interface::into_raw` / `from_raw`; interface pointers are ABI-stable.
///
/// The device is replaced after a device-lost event. Compare [`shared_d3d11_device_generation`]
/// against the value observed when you created your resources to detect that.
///
/// Multithread protection is enabled on the returned device, so decoder threads may use it
/// concurrently with the renderer.
pub fn shared_d3d11_device() -> Option<ID3D11Device061> {
    let guard = SHARED_DEVICE.read().ok()?;
    let device = guard.as_ref()?.clone();
    // Both crate versions describe the same COM interface; transferring one owned reference
    // keeps the device alive for the returned value.
    Some(unsafe { windows_061::core::Interface::from_raw(device.into_raw()) })
}

/// A counter that increases every time [`shared_d3d11_device`] starts returning a new device.
pub fn shared_d3d11_device_generation() -> u64 {
    GENERATION.load(Ordering::SeqCst)
}
