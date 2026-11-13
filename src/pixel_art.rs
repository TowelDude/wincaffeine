use std::mem::zeroed;
use std::ptr::null_mut;
use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateDIBSection, DeleteObject, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, DestroyIcon, HICON, ICONINFO,
};

use crate::pixel_data::*;

pub struct IconSet {
    pub active_16: HICON,
    pub inactive_16: HICON,
    pub active_32: HICON,
    pub inactive_32: HICON,
    pub active_48: HICON,
    pub inactive_48: HICON,
}

impl IconSet {
    pub fn new() -> Self {
        unsafe {
            Self {
                active_16: create_icon_from_rgba(16, 16, &ACTIVE_16_RGBA),
                inactive_16: create_icon_from_rgba(16, 16, &INACTIVE_16_RGBA),
                active_32: create_icon_from_rgba(32, 32, &ACTIVE_32_RGBA),
                inactive_32: create_icon_from_rgba(32, 32, &INACTIVE_32_RGBA),
                active_48: create_icon_from_rgba(48, 48, &ACTIVE_48_RGBA),
                inactive_48: create_icon_from_rgba(48, 48, &INACTIVE_48_RGBA),
            }
        }
    }

    pub fn get_tray_icon(&self, active: bool, high_dpi: bool) -> HICON {
        if high_dpi {
            if active { self.active_32 } else { self.inactive_32 }
        } else {
            if active { self.active_16 } else { self.inactive_16 }
        }
    }
}

impl Drop for IconSet {
    fn drop(&mut self) {
        unsafe {
            if !self.active_16.is_null() { DestroyIcon(self.active_16); }
            if !self.inactive_16.is_null() { DestroyIcon(self.inactive_16); }
            if !self.active_32.is_null() { DestroyIcon(self.active_32); }
            if !self.inactive_32.is_null() { DestroyIcon(self.inactive_32); }
            if !self.active_48.is_null() { DestroyIcon(self.active_48); }
            if !self.inactive_48.is_null() { DestroyIcon(self.inactive_48); }
        }
    }
}

pub unsafe fn create_icon_from_rgba(width: i32, height: i32, rgba: &[u8]) -> HICON {
    let mut bmi: BITMAPINFO = unsafe { zeroed() };
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height; // Top-down DIB
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    let hdc = unsafe { GetDC(null_mut()) };
    let mut bits: *mut core::ffi::c_void = null_mut();
    let hbm_color = unsafe {
        CreateDIBSection(
            hdc,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits,
            null_mut(),
            0,
        )
    };

    if !bits.is_null() && !hbm_color.is_null() {
        let dest = unsafe {
            std::slice::from_raw_parts_mut(bits as *mut u8, (width * height * 4) as usize)
        };
        for i in 0..(width * height) as usize {
            let r = rgba[i * 4];
            let g = rgba[i * 4 + 1];
            let b = rgba[i * 4 + 2];
            let a = rgba[i * 4 + 3];

            // Standard Windows 32-bit icon uses BGRA with pre-multiplied alpha for clean blending
            let alpha = a as u16;
            let pr = ((r as u16 * alpha) / 255) as u8;
            let pg = ((g as u16 * alpha) / 255) as u8;
            let pb = ((b as u16 * alpha) / 255) as u8;

            dest[i * 4] = pb;
            dest[i * 4 + 1] = pg;
            dest[i * 4 + 2] = pr;
            dest[i * 4 + 3] = a;
        }
    }

    // 1-bit monochrome mask (icon transparency mask)
    let hbm_mask = unsafe { CreateBitmap(width, height, 1, 1, null_mut()) };

    let mut icon_info: ICONINFO = unsafe { zeroed() };
    icon_info.fIcon = 1; // 1 = Icon
    icon_info.hbmColor = hbm_color;
    icon_info.hbmMask = hbm_mask;

    let icon = unsafe { CreateIconIndirect(&icon_info) };

    unsafe {
        DeleteObject(hbm_color);
        DeleteObject(hbm_mask);
        ReleaseDC(null_mut(), hdc);
    }

    icon
}
