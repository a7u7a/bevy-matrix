//! Local FFI for [hzeller/rpi-rgb-led-matrix](https://github.com/hzeller/rpi-rgb-led-matrix) C APIs
//! not wrapped by `rpi-led-matrix` (notably `led_canvas_set_pixels`).

use rpi_led_matrix::LedCanvas;
use std::ffi::c_int;

/// Matches C `struct Color` / `LedColor` layout (three u8 channels).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MatrixRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[link(name = "rgbmatrix")]
unsafe extern "C" {
    fn led_canvas_set_pixels(
        canvas: *mut std::ffi::c_void,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        colors: *const MatrixRgb,
    );
}

/// Read the raw canvas pointer from [`LedCanvas`].
///
/// # Safety
/// `rpi-led-matrix` 0.2.x implements `LedCanvas` as a single raw pointer field; this copies those
/// bytes. If the crate layout changes, the `size_of` assert fails at compile time.
#[inline]
pub(crate) unsafe fn led_canvas_as_void_ptr(canvas: &LedCanvas) -> *mut std::ffi::c_void {
    const _: () =
        assert!(core::mem::size_of::<LedCanvas>() == core::mem::size_of::<*mut std::ffi::c_void>());
    unsafe { core::mem::transmute_copy(canvas) }
}

/// Row-major RGB rectangle `(x, y)` .. `(x+width, y+height)` in one library call.
///
/// `colors` must hold at least `width * height` entries; `canvas` must be a live handle from
/// `rpi_led_matrix` (e.g. offscreen canvas from [`LedMatrix`](rpi_led_matrix::LedMatrix)).
pub(crate) fn canvas_set_pixels_bulk(
    canvas: &LedCanvas,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    colors: &[MatrixRgb],
) {
    let need = (width as usize).saturating_mul(height as usize);
    debug_assert!(colors.len() >= need);
    // SAFETY: FFI + newtype pointer extraction; invariants documented above.
    unsafe {
        let ptr = led_canvas_as_void_ptr(canvas);
        led_canvas_set_pixels(
            ptr,
            x as c_int,
            y as c_int,
            width as c_int,
            height as c_int,
            colors.as_ptr(),
        );
    }
}
