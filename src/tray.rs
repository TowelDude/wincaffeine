use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, POINT};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_TIP, NIIF_INFO, NIM_ADD, NIM_DELETE,
    NIM_MODIFY, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::pixel_art::IconSet;
use crate::power::PowerManager;
use crate::service;

pub const WM_APP_TRAY: u32 = WM_APP + 100;

pub const IDM_STATUS: usize = 1000;
pub const IDM_DEACTIVATE: usize = 1001;
pub const IDM_KEEP_15: usize = 1002;
pub const IDM_KEEP_30: usize = 1003;
pub const IDM_KEEP_60: usize = 1004;
pub const IDM_KEEP_INDEFINITE: usize = 1005;
pub const IDM_CUSTOM: usize = 1006;
pub const IDM_AUTOSTART: usize = 1007;
pub const IDM_SETUP: usize = 1008;
pub const IDM_ABOUT: usize = 1009;
pub const IDM_EXIT: usize = 1010;

pub struct TrayIcon {
    hwnd: HWND,
    high_dpi: bool,
    added: bool,
}

impl TrayIcon {
    pub fn new(hwnd: HWND, icons: &IconSet, power: &PowerManager) -> Self {
        let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let high_dpi = screen_h > 1200; // heuristic or 100%+ scale

        let mut tray = Self {
            hwnd,
            high_dpi,
            added: false,
        };
        tray.added = tray.init_tray_icon(icons, power);
        tray
    }

    pub fn is_added(&self) -> bool {
        self.added
    }

    pub fn init_tray_icon(&mut self, icons: &IconSet, power: &PowerManager) -> bool {
        let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.uCallbackMessage = WM_APP_TRAY;
        nid.hIcon = icons.get_tray_icon(power.is_active(), self.high_dpi);

        let tip = power.tooltip_text();
        unsafe {
            copy_to_wide_ptr(&tip, std::ptr::addr_of_mut!(nid.szTip) as *mut u16, 128);
            let ok = Shell_NotifyIconW(NIM_ADD, &nid) != 0;
            self.added = ok;
            ok
        }
    }

    pub fn update(&mut self, icons: &IconSet, power: &PowerManager) {
        let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_TIP;
        nid.hIcon = icons.get_tray_icon(power.is_active(), self.high_dpi);

        let tip = power.tooltip_text();
        unsafe {
            copy_to_wide_ptr(&tip, std::ptr::addr_of_mut!(nid.szTip) as *mut u16, 128);
            if Shell_NotifyIconW(NIM_MODIFY, &nid) == 0 {
                // Tray icon missing (e.g. explorer restart or late boot) - recreate
                self.init_tray_icon(icons, power);
            }
        }
    }

    pub fn show_balloon(&mut self, icons: &IconSet, title: &str, message: &str, active: bool) {
        let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_ICON | NIF_INFO;
        nid.hIcon = icons.get_tray_icon(active, self.high_dpi);
        nid.dwInfoFlags = NIIF_INFO;

        unsafe {
            copy_to_wide_ptr(
                title,
                std::ptr::addr_of_mut!(nid.szInfoTitle) as *mut u16,
                64,
            );
            copy_to_wide_ptr(
                message,
                std::ptr::addr_of_mut!(nid.szInfo) as *mut u16,
                256,
            );
            Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    pub fn show_context_menu(&self, power: &PowerManager) {
        unsafe {
            let hmenu = CreatePopupMenu();

            // Status header
            let header_str = power.menu_header_text();
            append_menu_item(hmenu, MF_STRING | MF_GRAYED, IDM_STATUS, &header_str);

            append_separator(hmenu);

            // Allow sleep / Deactivate
            let deact_flags = if !power.is_active() {
                MF_STRING | MF_CHECKED
            } else {
                MF_STRING
            };
            append_menu_item(
                hmenu,
                deact_flags,
                IDM_DEACTIVATE,
                "Allow Sleep (Turn Off)",
            );

            append_separator(hmenu);

            // Keep Awake durations
            let is_15 = match power.mode() {
                crate::power::CaffeineMode::Timed { total_secs, .. } => total_secs == 15 * 60,
                _ => false,
            };
            let is_30 = match power.mode() {
                crate::power::CaffeineMode::Timed { total_secs, .. } => total_secs == 30 * 60,
                _ => false,
            };
            let is_60 = match power.mode() {
                crate::power::CaffeineMode::Timed { total_secs, .. } => total_secs == 60 * 60,
                _ => false,
            };
            let is_indefinite = matches!(power.mode(), crate::power::CaffeineMode::Indefinite);

            let flag_15 = if is_15 { MF_STRING | MF_CHECKED } else { MF_STRING };
            let flag_30 = if is_30 { MF_STRING | MF_CHECKED } else { MF_STRING };
            let flag_60 = if is_60 { MF_STRING | MF_CHECKED } else { MF_STRING };
            let flag_indef = if is_indefinite {
                MF_STRING | MF_CHECKED
            } else {
                MF_STRING
            };

            append_menu_item(hmenu, flag_15, IDM_KEEP_15, "15 Minutes");
            append_menu_item(hmenu, flag_30, IDM_KEEP_30, "30 Minutes");
            append_menu_item(hmenu, flag_60, IDM_KEEP_60, "60 Minutes");
            append_menu_item(
                hmenu,
                flag_indef,
                IDM_KEEP_INDEFINITE,
                "Until Turned Off",
            );
            append_menu_item(
                hmenu,
                MF_STRING,
                IDM_CUSTOM,
                "Custom Duration or Time...",
            );

            append_separator(hmenu);

            // Windows Autostart / Background Service registration
            let autostart_flags = if service::is_autostart_enabled() {
                MF_STRING | MF_CHECKED
            } else {
                MF_STRING
            };
            append_menu_item(
                hmenu,
                autostart_flags,
                IDM_AUTOSTART,
                "Start with Windows (Service)",
            );
            append_menu_item(hmenu, MF_STRING, IDM_SETUP, "Service Setup & Install...");
            append_menu_item(hmenu, MF_STRING, IDM_ABOUT, "About Caffeine...");

            append_separator(hmenu);

            append_menu_item(hmenu, MF_STRING, IDM_EXIT, "Exit");

            let mut pt: POINT = std::mem::zeroed();
            GetCursorPos(&mut pt);

            SetForegroundWindow(self.hwnd);
            TrackPopupMenuEx(
                hmenu,
                TPM_RIGHTBUTTON | TPM_NONOTIFY,
                pt.x,
                pt.y,
                self.hwnd,
                null_mut(),
            );
            PostMessageW(self.hwnd, WM_NULL, 0, 0);

            DestroyMenu(hmenu);
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = 1;
            Shell_NotifyIconW(NIM_DELETE, &nid);
        }
    }
}

unsafe fn copy_to_wide_ptr(src: &str, dest: *mut u16, max_len: usize) {
    let wide: Vec<u16> = src.encode_utf16().collect();
    let copy_len = wide.len().min(max_len.saturating_sub(1));
    unsafe {
        for i in 0..copy_len {
            std::ptr::write_unaligned(dest.add(i), wide[i]);
        }
        std::ptr::write_unaligned(dest.add(copy_len), 0);
    }
}

unsafe fn append_menu_item(hmenu: HMENU, flags: u32, id: usize, text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        AppendMenuW(hmenu, flags, id, wide.as_ptr());
    }
}

unsafe fn append_separator(hmenu: HMENU) {
    unsafe {
        AppendMenuW(hmenu, MF_SEPARATOR, 0, null_mut());
    }
}
