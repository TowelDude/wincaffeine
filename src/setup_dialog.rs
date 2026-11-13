use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, DeleteObject, DrawTextW, EndPaint, SelectObject, SetBkMode,
    SetTextColor, COLOR_BTNFACE, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_WORDBREAK, HFONT,
    PAINTSTRUCT, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, VK_ESCAPE, VK_RETURN};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::pixel_art::IconSet;
use crate::service;

const BM_SETCHECK: u32 = 0x00F1;
const BM_GETCHECK: u32 = 0x00F0;
const BST_CHECKED: usize = 1;

const ID_CHK_AUTOSTART: usize = 201;
const ID_CHK_COPY_APPDATA: usize = 202;
const ID_BTN_INSTALL: usize = 1;
const ID_BTN_PORTABLE: usize = 203;
const ID_BTN_UNINSTALL: usize = 204;
const ID_BTN_CANCEL: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupAction {
    Install { autostart: bool, copy_to_appdata: bool },
    RunPortably,
    Uninstall,
    Cancel,
}

struct SetupState {
    icons: *const IconSet,
    hfont_title: HFONT,
    hfont_ui: HFONT,
    hfont_bold: HFONT,
    h_chk_autostart: HWND,
    h_chk_copy: HWND,
    h_btn_install: HWND,
    h_btn_portable: HWND,
    h_btn_uninstall: HWND,
    h_btn_cancel: HWND,
    h_label_status: HWND,
    action: SetupAction,
}

pub fn show_setup_dialog(parent: HWND, icons: &IconSet) -> SetupAction {
    unsafe {
        let class_name: Vec<u16> = "CaffeineSetupDialog\0".encode_utf16().collect();
        let hinstance = GetModuleHandleW(null_mut());

        let mut wndclass: WNDCLASSW = std::mem::zeroed();
        wndclass.lpfnWndProc = Some(setup_wnd_proc);
        wndclass.hInstance = hinstance as _;
        wndclass.lpszClassName = class_name.as_ptr();
        wndclass.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        wndclass.hbrBackground = (COLOR_BTNFACE + 1) as _;

        RegisterClassW(&wndclass);

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let win_w = 460;
        let win_h = 420;
        let win_x = (screen_w - win_w) / 2;
        let win_y = (screen_h - win_h) / 2;

        let title: Vec<u16> = "Caffeine - Service Setup & Installation\0"
            .encode_utf16()
            .collect();

        let mut state = SetupState {
            icons: icons as *const IconSet,
            hfont_title: create_font("Segoe UI", 18, 700),
            hfont_ui: create_font("Segoe UI", 14, 400),
            hfont_bold: create_font("Segoe UI", 14, 600),
            h_chk_autostart: null_mut(),
            h_chk_copy: null_mut(),
            h_btn_install: null_mut(),
            h_btn_portable: null_mut(),
            h_btn_uninstall: null_mut(),
            h_btn_cancel: null_mut(),
            h_label_status: null_mut(),
            action: SetupAction::Cancel,
        };

        if !parent.is_null() {
            EnableWindow(parent, 0);
        }

        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME | WS_EX_TOPMOST,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
            win_x,
            win_y,
            win_w,
            win_h,
            parent,
            null_mut(),
            hinstance as _,
            &mut state as *mut SetupState as *const _,
        );

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if msg.message == WM_KEYDOWN {
                if msg.wParam == VK_RETURN as usize {
                    SendMessageW(hwnd, WM_COMMAND, ID_BTN_INSTALL, 0);
                    continue;
                } else if msg.wParam == VK_ESCAPE as usize {
                    SendMessageW(hwnd, WM_COMMAND, ID_BTN_CANCEL, 0);
                    continue;
                }
            }

            if IsDialogMessageW(hwnd, &mut msg) == 0 {
                TranslateMessage(&mut msg);
                DispatchMessageW(&mut msg);
            }

            if IsWindow(hwnd) == 0 {
                break;
            }
        }

        if !parent.is_null() {
            EnableWindow(parent, 1);
            SetForegroundWindow(parent);
        }

        DeleteObject(state.hfont_title);
        DeleteObject(state.hfont_ui);
        DeleteObject(state.hfont_bold);

        state.action
    }
}

unsafe fn create_font(name: &str, height: i32, weight: i32) -> HFONT {
    let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        CreateFontW(
            -height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
            wide_name.as_ptr(),
        )
    }
}

unsafe extern "system" fn setup_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SetupState };

    match msg {
        WM_CREATE => {
            let cs = lparam as *const CREATESTRUCTW;
            unsafe {
                let state = (*cs).lpCreateParams as *mut SetupState;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, (state as usize) as _);
                init_setup_controls(hwnd, &mut *state);
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            if !state_ptr.is_null() {
                unsafe {
                    let state = &*state_ptr;
                    let icons = &*state.icons;
                    // Draw 48x48 mug pixel art
                    DrawIconEx(
                        hdc,
                        24,
                        16,
                        icons.active_48,
                        48,
                        48,
                        0,
                        null_mut(),
                        DI_NORMAL,
                    );

                    // Title
                    SelectObject(hdc, state.hfont_title);
                    SetBkMode(hdc, TRANSPARENT as _);
                    SetTextColor(hdc, 0x001B1B1B);
                    let title_text: Vec<u16> =
                        "Caffeine Service Setup\0".encode_utf16().collect();
                    let mut rc_title = RECT {
                        left: 88,
                        top: 16,
                        right: 440,
                        bottom: 40,
                    };
                    DrawTextW(
                        hdc,
                        title_text.as_ptr(),
                        -1,
                        &mut rc_title,
                        DT_LEFT | DT_SINGLELINE | DT_NOPREFIX,
                    );

                    // Subtitle
                    SelectObject(hdc, state.hfont_ui);
                    SetTextColor(hdc, 0x00666666);
                    let sub_text: Vec<u16> =
                        "Windows Background Service & Startup Registration\0"
                            .encode_utf16()
                            .collect();
                    let mut rc_sub = RECT {
                        left: 88,
                        top: 42,
                        right: 440,
                        bottom: 65,
                    };
                    DrawTextW(
                        hdc,
                        sub_text.as_ptr(),
                        -1,
                        &mut rc_sub,
                        DT_LEFT | DT_SINGLELINE | DT_NOPREFIX,
                    );

                    // Description text
                    let desc_text: Vec<u16> =
                        "Caffeine keeps your display and PC awake with zero bloat.\nInstalling registers Caffeine as a background service so it starts silently in your system tray on logon."
                            .encode_utf16()
                            .chain(std::iter::once(0))
                            .collect();
                    let mut rc_desc = RECT {
                        left: 24,
                        top: 75,
                        right: 430,
                        bottom: 130,
                    };
                    DrawTextW(
                        hdc,
                        desc_text.as_ptr(),
                        -1,
                        &mut rc_desc,
                        DT_LEFT | DT_WORDBREAK | DT_NOPREFIX,
                    );
                }
            }

            unsafe {
                EndPaint(hwnd, &ps);
            }
            0
        }
        WM_COMMAND => {
            if state_ptr.is_null() {
                return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
            }
            let state = unsafe { &mut *state_ptr };
            let id = wparam & 0xFFFF;

            match id {
                ID_BTN_INSTALL => unsafe {
                    let autostart =
                        SendMessageW(state.h_chk_autostart, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                    let copy =
                        SendMessageW(state.h_chk_copy, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                    state.action = SetupAction::Install {
                        autostart,
                        copy_to_appdata: copy,
                    };
                    DestroyWindow(hwnd);
                },
                ID_BTN_PORTABLE => unsafe {
                    state.action = SetupAction::RunPortably;
                    DestroyWindow(hwnd);
                },
                ID_BTN_UNINSTALL => unsafe {
                    state.action = SetupAction::Uninstall;
                    DestroyWindow(hwnd);
                },
                ID_BTN_CANCEL => unsafe {
                    state.action = SetupAction::Cancel;
                    DestroyWindow(hwnd);
                },
                _ => {}
            }
            0
        }
        WM_CLOSE => {
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

unsafe fn init_setup_controls(hwnd: HWND, state: &mut SetupState) {
    let hinst = unsafe { GetModuleHandleW(null_mut()) as isize };

    let is_installed = service::is_currently_installed();
    let is_autostart = service::is_autostart_enabled();

    // Checkboxes
    state.h_chk_autostart = unsafe {
        create_control(
            "BUTTON",
            "Start automatically with Windows (Background Service)",
            WS_CHILD | WS_VISIBLE | (BS_AUTOCHECKBOX as u32),
            24,
            135,
            400,
            24,
            hwnd,
            ID_CHK_AUTOSTART,
            hinst,
            state.hfont_bold,
        )
    };
    unsafe {
        SendMessageW(state.h_chk_autostart, BM_SETCHECK, BST_CHECKED, 0);
    }

    state.h_chk_copy = unsafe {
        create_control(
            "BUTTON",
            "Install executable to %LOCALAPPDATA%\\Caffeine",
            WS_CHILD | WS_VISIBLE | (BS_AUTOCHECKBOX as u32),
            24,
            165,
            400,
            24,
            hwnd,
            ID_CHK_COPY_APPDATA,
            hinst,
            state.hfont_ui,
        )
    };
    unsafe {
        SendMessageW(state.h_chk_copy, BM_SETCHECK, BST_CHECKED, 0);
    }

    // Status label
    let status_str = format!(
        "Status: {}\nStartup Service: {}",
        if is_installed {
            "Installed in LocalAppData"
        } else {
            "Running portable / not installed"
        },
        if is_autostart {
            "Enabled (starts on boot)"
        } else {
            "Not registered"
        }
    );

    state.h_label_status = unsafe {
        create_control(
            "STATIC",
            &status_str,
            WS_CHILD | WS_VISIBLE,
            24,
            205,
            400,
            45,
            hwnd,
            998,
            hinst,
            state.hfont_ui,
        )
    };

    // Action buttons
    state.h_btn_install = unsafe {
        create_control(
            "BUTTON",
            "Install as Background Service",
            WS_CHILD | WS_VISIBLE | (BS_DEFPUSHBUTTON as u32),
            24,
            265,
            220,
            34,
            hwnd,
            ID_BTN_INSTALL,
            hinst,
            state.hfont_bold,
        )
    };

    state.h_btn_portable = unsafe {
        create_control(
            "BUTTON",
            "Run Portably Once",
            WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32),
            255,
            265,
            170,
            34,
            hwnd,
            ID_BTN_PORTABLE,
            hinst,
            state.hfont_ui,
        )
    };

    state.h_btn_uninstall = unsafe {
        create_control(
            "BUTTON",
            "Uninstall Service",
            WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32),
            24,
            315,
            140,
            32,
            hwnd,
            ID_BTN_UNINSTALL,
            hinst,
            state.hfont_ui,
        )
    };
    if !is_installed && !is_autostart {
        unsafe {
            EnableWindow(state.h_btn_uninstall, 0);
        }
    }

    state.h_btn_cancel = unsafe {
        create_control(
            "BUTTON",
            "Close / Cancel",
            WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32),
            315,
            315,
            110,
            32,
            hwnd,
            ID_BTN_CANCEL,
            hinst,
            state.hfont_ui,
        )
    };
}

unsafe fn create_control(
    class: &str,
    text: &str,
    style: u32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    parent: HWND,
    id: usize,
    hinst: isize,
    font: HFONT,
) -> HWND {
    let wide_class: Vec<u16> = class.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_text: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

    let hwnd = unsafe {
        CreateWindowExW(
            0,
            wide_class.as_ptr(),
            wide_text.as_ptr(),
            style,
            x,
            y,
            w,
            h,
            parent,
            id as _,
            hinst as _,
            null_mut(),
        )
    };

    if !font.is_null() {
        unsafe {
            SendMessageW(hwnd, WM_SETFONT, font as usize, 1);
        }
    }

    hwnd
}
