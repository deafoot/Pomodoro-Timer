use windows::Win32::Foundation::{COLORREF, HWND};
use windows::Win32::UI::Controls::Dialogs::{ChooseColorW, CC_ANYCOLOR, CC_FULLOPEN, CC_RGBINIT, CHOOSECOLORW};

use crate::color::Color;

pub fn pick_color(hwnd: HWND, initial: Color, custom: &mut [u32; 16]) -> Option<Color> {
    let initial_ref = COLORREF(
        ((initial.r * 255.0).round() as u32) | (((initial.g * 255.0).round() as u32) << 8) | (((initial.b * 255.0).round() as u32) << 16),
    );
    let mut data = CHOOSECOLORW {
        lStructSize: std::mem::size_of::<CHOOSECOLORW>() as u32,
        hwndOwner: hwnd,
        rgbResult: initial_ref,
        lpCustColors: custom.as_mut_ptr() as *mut COLORREF,
        Flags: CC_RGBINIT | CC_FULLOPEN | CC_ANYCOLOR,
        ..Default::default()
    };
    let ok = unsafe { ChooseColorW(&mut data).as_bool() };
    if !ok {
        return None;
    }
    let value = data.rgbResult.0;
    Some(Color::rgb(
        (value & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        ((value >> 16) & 0xFF) as u8,
    ))
}

