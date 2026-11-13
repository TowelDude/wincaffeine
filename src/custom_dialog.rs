use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, DeleteObject, DrawTextW, EndPaint, SelectObject, SetBkMode,
    SetTextColor, COLOR_BTNFACE, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, HFONT, PAINTSTRUCT,
    TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{EnableWindow, VK_ESCAPE, VK_RETURN};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::pixel_art::IconSet;
use crate::power::get_local_time;

const BM_SETCHECK: u32 = 0x00F1;
const BM_GETCHECK: u32 = 0x00F0;
const BST_UNCHECKED: usize = 0;
const BST_CHECKED: usize = 1;

const SS_CENTER: u32 = 0x00000001;

const ID_RADIO_DURATION: usize = 101;
const ID_EDIT_DURATION: usize = 102;
const ID_RADIO_MINS: usize = 103;
const ID_RADIO_HOURS: usize = 104;

const ID_BTN_P15: usize = 105;
const ID_BTN_P30: usize = 106;
const ID_BTN_P60: usize = 107;
const ID_BTN_P120: usize = 108;
const ID_BTN_P240: usize = 109;

const ID_RADIO_TIME: usize = 110;
const ID_EDIT_TIME: usize = 111;

const ID_LABEL_PREVIEW: usize = 112;

const ID_BTN_OK: usize = 1;
const ID_BTN_CANCEL: usize = 2;

#[derive(Debug, Clone, Copy)]
pub enum CustomTimerChoice {
    DurationSeconds(u64),
    SpecificTime(u16, u16),
}

struct DialogState {
    icons: *const IconSet,
    hfont_title: HFONT,
    hfont_ui: HFONT,
    hfont_bold: HFONT,
    h_radio_dur: HWND,
    h_edit_dur: HWND,
    h_radio_mins: HWND,
    h_radio_hours: HWND,
    h_btn_p15: HWND,
    h_btn_p30: HWND,
    h_btn_p60: HWND,
    h_btn_p120: HWND,
    h_btn_p240: HWND,
    h_radio_time: HWND,
    h_edit_time: HWND,
    h_label_preview: HWND,
    h_btn_ok: HWND,
    h_btn_cancel: HWND,
    result: Option<CustomTimerChoice>,
}

pub fn show_custom_timer_dialog(parent: HWND, icons: &IconSet) -> Option<CustomTimerChoice> {
    unsafe {
        let class_name: Vec<u16> = "CaffeineCustomDialog\0".encode_utf16().collect();
        let hinstance = GetModuleHandleW(null_mut());

        let mut wndclass: WNDCLASSW = std::mem::zeroed();
        wndclass.lpfnWndProc = Some(dialog_wnd_proc);
        wndclass.hInstance = hinstance as _;
        wndclass.lpszClassName = class_name.as_ptr();
        wndclass.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        wndclass.hbrBackground = (COLOR_BTNFACE + 1) as _;

        RegisterClassW(&wndclass);

        // Center on screen
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let win_w = 420;
        let win_h = 430;
        let win_x = (screen_w - win_w) / 2;
        let win_y = (screen_h - win_h) / 2;

        let title: Vec<u16> = "Caffeine - Custom Timer\0".encode_utf16().collect();

        let mut state = DialogState {
            icons: icons as *const IconSet,
            hfont_title: create_font("Segoe UI", 18, 700),
            hfont_ui: create_font("Segoe UI", 14, 400),
            hfont_bold: create_font("Segoe UI", 14, 600),
            h_radio_dur: null_mut(),
            h_edit_dur: null_mut(),
            h_radio_mins: null_mut(),
            h_radio_hours: null_mut(),
            h_btn_p15: null_mut(),
            h_btn_p30: null_mut(),
            h_btn_p60: null_mut(),
            h_btn_p120: null_mut(),
            h_btn_p240: null_mut(),
            h_radio_time: null_mut(),
            h_edit_time: null_mut(),
            h_label_preview: null_mut(),
            h_btn_ok: null_mut(),
            h_btn_cancel: null_mut(),
            result: None,
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
            &mut state as *mut DialogState as *const _,
        );

        // Message loop for modal dialog
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if msg.message == WM_KEYDOWN {
                if msg.wParam == VK_RETURN as usize {
                    SendMessageW(hwnd, WM_COMMAND, ID_BTN_OK, 0);
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

        state.result
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
            1, // DEFAULT_CHARSET
            0,
            0,
            0,
            0,
            wide_name.as_ptr(),
        )
    }
}

unsafe fn set_window_text_str(hwnd: HWND, text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        SetWindowTextW(hwnd, wide.as_ptr());
    }
}

unsafe fn get_window_text_str(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let len = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    if len > 0 {
        String::from_utf16_lossy(&buf[..len as usize])
    } else {
        String::new()
    }
}

unsafe extern "system" fn dialog_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut DialogState };

    match msg {
        WM_CREATE => {
            let cs = lparam as *const CREATESTRUCTW;
            unsafe {
                let state = (*cs).lpCreateParams as *mut DialogState;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, (state as usize) as _);
                init_dialog_controls(hwnd, &mut *state);
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
                    // Draw 48x48 active coffee mug pixel art at (20, 16)
                    DrawIconEx(
                        hdc,
                        20,
                        16,
                        icons.active_48,
                        48,
                        48,
                        0,
                        null_mut(),
                        DI_NORMAL,
                    );

                    // Draw header text
                    SelectObject(hdc, state.hfont_title);
                    SetBkMode(hdc, TRANSPARENT as _);
                    SetTextColor(hdc, 0x001B1B1B); // Dark near-black
                    let title_text: Vec<u16> = "Caffeine Custom Timer\0".encode_utf16().collect();
                    let mut rc_title = RECT {
                        left: 80,
                        top: 16,
                        right: 400,
                        bottom: 40,
                    };
                    DrawTextW(
                        hdc,
                        title_text.as_ptr(),
                        -1,
                        &mut rc_title,
                        DT_LEFT | DT_SINGLELINE | DT_NOPREFIX,
                    );

                    SelectObject(hdc, state.hfont_ui);
                    SetTextColor(hdc, 0x00666666); // Subtitle gray
                    let sub_text: Vec<u16> =
                        "Select duration or exact time to stay awake\0".encode_utf16().collect();
                    let mut rc_sub = RECT {
                        left: 80,
                        top: 42,
                        right: 400,
                        bottom: 65,
                    };
                    DrawTextW(
                        hdc,
                        sub_text.as_ptr(),
                        -1,
                        &mut rc_sub,
                        DT_LEFT | DT_SINGLELINE | DT_NOPREFIX,
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
            let notify_code = (wparam >> 16) & 0xFFFF;

            match id {
                ID_RADIO_DURATION => unsafe {
                    SendMessageW(state.h_radio_dur, BM_SETCHECK, BST_CHECKED, 0);
                    SendMessageW(state.h_radio_time, BM_SETCHECK, BST_UNCHECKED, 0);
                    update_preview(state);
                },
                ID_RADIO_TIME => unsafe {
                    SendMessageW(state.h_radio_dur, BM_SETCHECK, BST_UNCHECKED, 0);
                    SendMessageW(state.h_radio_time, BM_SETCHECK, BST_CHECKED, 0);
                    update_preview(state);
                },
                ID_RADIO_MINS => unsafe {
                    SendMessageW(state.h_radio_mins, BM_SETCHECK, BST_CHECKED, 0);
                    SendMessageW(state.h_radio_hours, BM_SETCHECK, BST_UNCHECKED, 0);
                    update_preview(state);
                },
                ID_RADIO_HOURS => unsafe {
                    SendMessageW(state.h_radio_mins, BM_SETCHECK, BST_UNCHECKED, 0);
                    SendMessageW(state.h_radio_hours, BM_SETCHECK, BST_CHECKED, 0);
                    update_preview(state);
                },
                ID_BTN_P15 => unsafe {
                    set_duration_preset(state, "15");
                },
                ID_BTN_P30 => unsafe {
                    set_duration_preset(state, "30");
                },
                ID_BTN_P60 => unsafe {
                    set_duration_preset(state, "60");
                },
                ID_BTN_P120 => unsafe {
                    set_duration_preset(state, "120");
                },
                ID_BTN_P240 => unsafe {
                    set_duration_preset(state, "240");
                },
                ID_EDIT_DURATION | ID_EDIT_TIME => {
                    if notify_code == EN_CHANGE as usize {
                        unsafe {
                            update_preview(state);
                        }
                    }
                }
                ID_BTN_OK => unsafe {
                    let is_dur = SendMessageW(state.h_radio_dur, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                    if is_dur {
                        let text = get_window_text_str(state.h_edit_dur);
                        let val: u64 = text.trim().parse().unwrap_or(30);
                        let is_hours = SendMessageW(state.h_radio_hours, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                        let seconds = if is_hours {
                            val * 3600
                        } else {
                            val * 60
                        };
                        state.result = Some(CustomTimerChoice::DurationSeconds(seconds.max(60)));
                    } else {
                        let text = get_window_text_str(state.h_edit_time);
                        if let Some((h, m)) = parse_hh_mm(&text) {
                            state.result = Some(CustomTimerChoice::SpecificTime(h, m));
                        } else {
                            let (cur_h, cur_m) = get_local_time();
                            let target_h = (cur_h + 1) % 24;
                            state.result = Some(CustomTimerChoice::SpecificTime(target_h, cur_m));
                        }
                    }
                    DestroyWindow(hwnd);
                },
                ID_BTN_CANCEL => unsafe {
                    state.result = None;
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

unsafe fn init_dialog_controls(hwnd: HWND, state: &mut DialogState) {
    let hinst = unsafe { GetModuleHandleW(null_mut()) as isize };

    // Section 1: Duration radio button
    state.h_radio_dur = unsafe {
        create_control(
            "BUTTON",
            "Keep screen awake for a duration:",
            WS_CHILD | WS_VISIBLE | (BS_AUTORADIOBUTTON as u32),
            20,
            80,
            280,
            24,
            hwnd,
            ID_RADIO_DURATION,
            hinst,
            state.hfont_bold,
        )
    };
    unsafe {
        SendMessageW(state.h_radio_dur, BM_SETCHECK, BST_CHECKED, 0);
    }

    // Number edit box
    state.h_edit_dur = unsafe {
        create_control(
            "EDIT",
            "45",
            WS_CHILD | WS_VISIBLE | WS_BORDER | (ES_NUMBER as u32) | (ES_CENTER as u32),
            45,
            110,
            60,
            26,
            hwnd,
            ID_EDIT_DURATION,
            hinst,
            state.hfont_ui,
        )
    };

    // Unit radios: Minutes / Hours
    state.h_radio_mins = unsafe {
        create_control(
            "BUTTON",
            "Minutes",
            WS_CHILD | WS_VISIBLE | (BS_AUTORADIOBUTTON as u32),
            115,
            112,
            75,
            22,
            hwnd,
            ID_RADIO_MINS,
            hinst,
            state.hfont_ui,
        )
    };
    unsafe {
        SendMessageW(state.h_radio_mins, BM_SETCHECK, BST_CHECKED, 0);
    }

    state.h_radio_hours = unsafe {
        create_control(
            "BUTTON",
            "Hours",
            WS_CHILD | WS_VISIBLE | (BS_AUTORADIOBUTTON as u32),
            195,
            112,
            70,
            22,
            hwnd,
            ID_RADIO_HOURS,
            hinst,
            state.hfont_ui,
        )
    };

    // Quick presets
    state.h_btn_p15 = unsafe { create_control("BUTTON", "+15m", WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32), 45, 145, 52, 24, hwnd, ID_BTN_P15, hinst, state.hfont_ui) };
    state.h_btn_p30 = unsafe { create_control("BUTTON", "+30m", WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32), 102, 145, 52, 24, hwnd, ID_BTN_P30, hinst, state.hfont_ui) };
    state.h_btn_p60 = unsafe { create_control("BUTTON", "+1h", WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32), 159, 145, 52, 24, hwnd, ID_BTN_P60, hinst, state.hfont_ui) };
    state.h_btn_p120 = unsafe { create_control("BUTTON", "+2h", WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32), 216, 145, 52, 24, hwnd, ID_BTN_P120, hinst, state.hfont_ui) };
    state.h_btn_p240 = unsafe { create_control("BUTTON", "+4h", WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32), 273, 145, 52, 24, hwnd, ID_BTN_P240, hinst, state.hfont_ui) };

    // Section 2: Specific Time radio button
    state.h_radio_time = unsafe {
        create_control(
            "BUTTON",
            "Keep screen awake until specific time:",
            WS_CHILD | WS_VISIBLE | (BS_AUTORADIOBUTTON as u32),
            20,
            195,
            320,
            24,
            hwnd,
            ID_RADIO_TIME,
            hinst,
            state.hfont_bold,
        )
    };

    // Specific time input (HH:MM)
    let (cur_h, cur_m) = get_local_time();
    let def_target_h = (cur_h + 1) % 24;
    let def_time_str = format!("{:02}:{:02}", def_target_h, cur_m);

    state.h_edit_time = unsafe {
        create_control(
            "EDIT",
            &def_time_str,
            WS_CHILD | WS_VISIBLE | WS_BORDER | (ES_CENTER as u32),
            45,
            225,
            75,
            26,
            hwnd,
            ID_EDIT_TIME,
            hinst,
            state.hfont_ui,
        )
    };

    let hint_str = format!("24-hr format (Current local time: {:02}:{:02})", cur_h, cur_m);
    unsafe {
        create_control(
            "STATIC",
            &hint_str,
            WS_CHILD | WS_VISIBLE,
            130,
            229,
            250,
            20,
            hwnd,
            999,
            hinst,
            state.hfont_ui,
        );
    }

    // Dynamic Preview Box
    state.h_label_preview = unsafe {
        create_control(
            "STATIC",
            "Preview: Calculating...",
            WS_CHILD | WS_VISIBLE | SS_CENTER,
            20,
            280,
            365,
            36,
            hwnd,
            ID_LABEL_PREVIEW,
            hinst,
            state.hfont_bold,
        )
    };

    // Action buttons
    state.h_btn_ok = unsafe {
        create_control(
            "BUTTON",
            "Activate Caffeine",
            WS_CHILD | WS_VISIBLE | (BS_DEFPUSHBUTTON as u32),
            80,
            335,
            140,
            34,
            hwnd,
            ID_BTN_OK,
            hinst,
            state.hfont_bold,
        )
    };

    state.h_btn_cancel = unsafe {
        create_control(
            "BUTTON",
            "Cancel",
            WS_CHILD | WS_VISIBLE | (BS_PUSHBUTTON as u32),
            235,
            335,
            100,
            34,
            hwnd,
            ID_BTN_CANCEL,
            hinst,
            state.hfont_ui,
        )
    };

    unsafe {
        update_preview(state);
    }
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

unsafe fn set_duration_preset(state: &mut DialogState, text: &str) {
    unsafe {
        SendMessageW(state.h_radio_dur, BM_SETCHECK, BST_CHECKED, 0);
        SendMessageW(state.h_radio_time, BM_SETCHECK, BST_UNCHECKED, 0);
        SendMessageW(state.h_radio_mins, BM_SETCHECK, BST_CHECKED, 0);
        SendMessageW(state.h_radio_hours, BM_SETCHECK, BST_UNCHECKED, 0);
        set_window_text_str(state.h_edit_dur, text);
        update_preview(state);
    }
}

unsafe fn update_preview(state: &mut DialogState) {
    let is_dur = unsafe { SendMessageW(state.h_radio_dur, BM_GETCHECK, 0, 0) == BST_CHECKED as isize };
    let (cur_h, cur_m) = get_local_time();

    if is_dur {
        let dur_text = unsafe { get_window_text_str(state.h_edit_dur) };
        let val: u64 = dur_text.trim().parse().unwrap_or(0);
        let is_hours = unsafe { SendMessageW(state.h_radio_hours, BM_GETCHECK, 0, 0) == BST_CHECKED as isize };

        let total_mins = if is_hours { val * 60 } else { val };

        if total_mins > 0 {
            let end_m = (cur_m as u64 + total_mins) % 60;
            let end_h = (cur_h as u64 + (cur_m as u64 + total_mins) / 60) % 24;
            let preview = format!(
                "Screen will stay awake for {} {} (until {:02}:{:02})",
                val,
                if is_hours {
                    if val == 1 { "hour" } else { "hours" }
                } else {
                    "minutes"
                },
                end_h,
                end_m
            );
            unsafe {
                set_window_text_str(state.h_label_preview, &preview);
            }
        } else {
            unsafe {
                set_window_text_str(state.h_label_preview, "Please enter a duration greater than 0");
            }
        }
    } else {
        let time_text = unsafe { get_window_text_str(state.h_edit_time) };
        if let Some((target_h, target_m)) = parse_hh_mm(&time_text) {
            let cur_total = (cur_h as i32) * 60 + (cur_m as i32);
            let target_total = (target_h as i32) * 60 + (target_m as i32);

            let diff = if target_total > cur_total {
                target_total - cur_total
            } else {
                (24 * 60 - cur_total) + target_total
            };

            let h_part = diff / 60;
            let m_part = diff % 60;
            let dur_str = if h_part > 0 && m_part > 0 {
                format!("{}h {}m", h_part, m_part)
            } else if h_part > 0 {
                format!("{} hours", h_part)
            } else {
                format!("{} minutes", m_part)
            };

            let preview = format!(
                "Screen will stay awake until {:02}:{:02} ({} from now)",
                target_h, target_m, dur_str
            );
            unsafe {
                set_window_text_str(state.h_label_preview, &preview);
            }
        } else {
            unsafe {
                set_window_text_str(state.h_label_preview, "Enter time as HH:MM (e.g. 18:30 or 22:00)");
            }
        }
    }
}

fn parse_hh_mm(s: &str) -> Option<(u16, u16)> {
    let s = s.trim();
    let parts: Vec<&str> = s.split(|c| c == ':' || c == '.' || c == ' ').filter(|p| !p.is_empty()).collect();
    if parts.len() == 2 {
        let h: u16 = parts[0].parse().ok()?;
        let m: u16 = parts[1].parse().ok()?;
        if h < 24 && m < 60 {
            return Some((h, m));
        }
    }
    None
}
