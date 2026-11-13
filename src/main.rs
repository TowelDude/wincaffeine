#![windows_subsystem = "windows"]

mod custom_dialog;
mod pixel_art;
mod pixel_data;
mod power;
mod service;
mod setup_dialog;
mod tray;

use std::ptr::null_mut;
use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use custom_dialog::{show_custom_timer_dialog, CustomTimerChoice};
use pixel_art::IconSet;
use power::PowerManager;
use setup_dialog::{show_setup_dialog, SetupAction};
use tray::{
    TrayIcon, IDM_ABOUT, IDM_AUTOSTART, IDM_CUSTOM, IDM_DEACTIVATE, IDM_EXIT, IDM_KEEP_15,
    IDM_KEEP_30, IDM_KEEP_60, IDM_KEEP_INDEFINITE, IDM_SETUP, WM_APP_TRAY,
};

const WM_APP_WAKEUP_INSTANCE: u32 = WM_APP + 200;
static mut WM_TASKBAR_CREATED: u32 = 0;

struct AppState {
    icons: IconSet,
    power: PowerManager,
    tray: Option<TrayIcon>,
}

fn main() {
    unsafe {
        SetProcessDPIAware();
    }

    let args: Vec<String> = std::env::args().collect();

    // Command-line handling
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" | "/?" => {
                show_message_box(
                    "Caffeine - Command Line Options",
                    "Usage: caffeine.exe [option]\n\n\
                     Options:\n  \
                     --tray             Run directly in system tray (silent startup)\n  \
                     --install          Install into %LOCALAPPDATA%\\Caffeine and register startup service\n  \
                     --uninstall        Unregister startup service and remove files\n  \
                     --status           Show current install and autostart status\n  \
                     --install-service  Register as a Windows Service (sc.exe)\n  \
                     --uninstall-service Unregister Windows Service\n  \
                     --help             Show this help message",
                );
                return;
            }
            "--install" => {
                match service::install_self(true) {
                    Ok(path) => {
                        show_message_box(
                            "Caffeine Installed",
                            &format!(
                                "Caffeine has been installed successfully!\n\nLocation: {}\nStartup Service: Enabled",
                                path.display()
                            ),
                        );
                    }
                    Err(e) => {
                        show_message_box("Install Error", &format!("Installation failed: {}", e));
                    }
                }
                return;
            }
            "--uninstall" => {
                match service::uninstall_self() {
                    Ok(()) => {
                        show_message_box(
                            "Caffeine Uninstalled",
                            "Caffeine startup service and registry entries have been removed.",
                        );
                    }
                    Err(e) => {
                        show_message_box("Uninstall Error", &format!("Uninstall failed: {}", e));
                    }
                }
                return;
            }
            "--install-service" => {
                let target_path = service::get_installed_exe_path();
                let cmd = format!(
                    "create CaffeineService binPath=\"{}\" start=auto DisplayName=\"Caffeine Keep-Alive Service\"",
                    target_path.display()
                );
                let res = std::process::Command::new("sc.exe")
                    .args(cmd.split_whitespace())
                    .output();
                match res {
                    Ok(out) => {
                        show_message_box(
                            "Service Install",
                            &format!("Service registration output:\n{}", String::from_utf8_lossy(&out.stdout)),
                        );
                    }
                    Err(e) => {
                        show_message_box("Service Install Error", &format!("Failed to execute sc.exe: {}", e));
                    }
                }
                return;
            }
            "--uninstall-service" => {
                let res = std::process::Command::new("sc.exe")
                    .args(["delete", "CaffeineService"])
                    .output();
                match res {
                    Ok(out) => {
                        show_message_box(
                            "Service Uninstall",
                            &format!("Service removal output:\n{}", String::from_utf8_lossy(&out.stdout)),
                        );
                    }
                    Err(e) => {
                        show_message_box("Service Uninstall Error", &format!("Failed to execute sc.exe: {}", e));
                    }
                }
                return;
            }
            "--status" => {
                let is_inst = service::is_currently_installed();
                let is_auto = service::is_autostart_enabled();
                show_message_box(
                    "Caffeine Status",
                    &format!(
                        "Installation status:\n  Installed in LocalAppData: {}\n  Startup Service (Autostart): {}",
                        if is_inst { "Yes" } else { "No" },
                        if is_auto { "Active" } else { "Disabled" }
                    ),
                );
                return;
            }
            "--tray" | "--silent" => {
                // Directly proceed to tray execution
            }
            _ => {}
        }
    }

    // Single-instance check
    let mutex_name: Vec<u16> = "Local\\CaffeineUtilityAppMutex\0".encode_utf16().collect();
    let _h_mutex = unsafe { CreateMutexW(null_mut(), 1, mutex_name.as_ptr()) };
    let already_running = unsafe { GetLastError() == ERROR_ALREADY_EXISTS };

    let class_name: Vec<u16> = "CaffeineTrayMsgWindow\0".encode_utf16().collect();
    let window_title: Vec<u16> = "CaffeineServiceMsgWindow\0".encode_utf16().collect();

    if already_running {
        unsafe {
            let existing_hwnd = FindWindowW(class_name.as_ptr(), window_title.as_ptr());
            if !existing_hwnd.is_null() {
                PostMessageW(existing_hwnd, WM_APP_WAKEUP_INSTANCE, 0, 0);
            }
        }
        return;
    }

    let icons = IconSet::new();

    // First-run UX: if not installed and not run with --tray, prompt user with Setup Dialog
    let is_tray_flag = args.iter().any(|a| a == "--tray" || a == "--silent");
    let is_installed = service::is_currently_installed();

    if !is_installed && !is_tray_flag {
        let action = show_setup_dialog(null_mut(), &icons);
        match action {
            SetupAction::Install {
                autostart,
                copy_to_appdata,
            } => {
                if copy_to_appdata {
                    if let Ok(installed_path) = service::install_self(autostart) {
                        // Launch installed executable with --tray and exit this installer
                        let _ = std::process::Command::new(installed_path)
                            .arg("--tray")
                            .spawn();
                        return;
                    }
                } else if autostart {
                    let _ = service::set_autostart(true, None);
                }
            }
            SetupAction::RunPortably => {
                // Continue running directly
            }
            SetupAction::Uninstall => {
                let _ = service::uninstall_self();
                return;
            }
            SetupAction::Cancel => {
                return;
            }
        }
    }

    // Register message window class
    unsafe {
        let hinstance = GetModuleHandleW(null_mut());
        let mut wndclass: WNDCLASSW = std::mem::zeroed();
        wndclass.lpfnWndProc = Some(main_wnd_proc);
        wndclass.hInstance = hinstance as _;
        wndclass.lpszClassName = class_name.as_ptr();

        RegisterClassW(&wndclass);

        let mut app_state = AppState {
            icons,
            power: PowerManager::new(),
            tray: None,
        };

        let taskbar_str: Vec<u16> = "TaskbarCreated\0".encode_utf16().collect();
        WM_TASKBAR_CREATED = RegisterWindowMessageW(taskbar_str.as_ptr());

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            hinstance as _,
            &mut app_state as *mut AppState as *const _,
        );

        if hwnd.is_null() {
            return;
        }

        // Initialize tray icon
        app_state.tray = Some(TrayIcon::new(hwnd, &app_state.icons, &app_state.power));

        // Start 1-second timer tick
        SetTimer(hwnd, 1, 1000, None);

        // Show welcome balloon on startup
        if let Some(tray) = &mut app_state.tray {
            tray.show_balloon(
                &app_state.icons,
                "Caffeine is Ready",
                "Click the coffee cup icon to keep your screen awake.",
                false,
            );
        }

        // Standard Win32 Message Loop
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&mut msg);
            DispatchMessageW(&mut msg);
        }

        // Clean up timer and tray on exit
        KillTimer(hwnd, 1);
        drop(app_state.tray.take());
    }
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState };

    match msg {
        WM_CREATE => {
            let cs = lparam as *const CREATESTRUCTW;
            unsafe {
                let state = (*cs).lpCreateParams as *mut AppState;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, (state as usize) as _);
            }
            0
        }
        m if m == unsafe { WM_TASKBAR_CREATED } && m != 0 => {
            if !state_ptr.is_null() {
                let state = unsafe { &mut *state_ptr };
                if let Some(tray) = &mut state.tray {
                    tray.init_tray_icon(&state.icons, &state.power);
                }
            }
            0
        }
        WM_TIMER => {
            if state_ptr.is_null() {
                return 0;
            }
            let state = unsafe { &mut *state_ptr };

            // Ensure tray icon is active if shell finished loading after app started
            if let Some(tray) = &mut state.tray {
                if !tray.is_added() {
                    tray.init_tray_icon(&state.icons, &state.power);
                }
            }

            let expired = state.power.tick();

            if expired {
                if let Some(tray) = &mut state.tray {
                    tray.update(&state.icons, &state.power);
                    tray.show_balloon(
                        &state.icons,
                        "Caffeine Timer Finished",
                        "Scheduled time elapsed. Screen sleep is now enabled.",
                        false,
                    );
                }
            } else if state.power.is_active() {
                // Update tooltip countdown
                if let Some(tray) = &mut state.tray {
                    tray.update(&state.icons, &state.power);
                }
            }
            0
        }
        WM_APP_TRAY => {
            if state_ptr.is_null() {
                return 0;
            }
            let state = unsafe { &mut *state_ptr };
            let event = lparam as u32;

            if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
                if let Some(tray) = &state.tray {
                    tray.show_context_menu(&state.power);
                }
            }
            0
        }
        WM_APP_WAKEUP_INSTANCE => {
            if state_ptr.is_null() {
                return 0;
            }
            let state = unsafe { &mut *state_ptr };
            if let Some(tray) = &mut state.tray {
                tray.init_tray_icon(&state.icons, &state.power);
                tray.show_context_menu(&state.power);
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
                IDM_DEACTIVATE => {
                    state.power.deactivate();
                    if let Some(tray) = &mut state.tray {
                        tray.update(&state.icons, &state.power);
                        tray.show_balloon(
                            &state.icons,
                            "Caffeine Deactivated",
                            "Screen sleep is now allowed naturally.",
                            false,
                        );
                    }
                }
                IDM_KEEP_15 => {
                    state.power.activate_timed_seconds(15 * 60);
                    if let Some(tray) = &mut state.tray {
                        tray.update(&state.icons, &state.power);
                        tray.show_balloon(
                            &state.icons,
                            "Caffeine Active (15m)",
                            "Screen will stay awake for 15 minutes.",
                            true,
                        );
                    }
                }
                IDM_KEEP_30 => {
                    state.power.activate_timed_seconds(30 * 60);
                    if let Some(tray) = &mut state.tray {
                        tray.update(&state.icons, &state.power);
                        tray.show_balloon(
                            &state.icons,
                            "Caffeine Active (30m)",
                            "Screen will stay awake for 30 minutes.",
                            true,
                        );
                    }
                }
                IDM_KEEP_60 => {
                    state.power.activate_timed_seconds(60 * 60);
                    if let Some(tray) = &mut state.tray {
                        tray.update(&state.icons, &state.power);
                        tray.show_balloon(
                            &state.icons,
                            "Caffeine Active (60m)",
                            "Screen will stay awake for 1 hour.",
                            true,
                        );
                    }
                }
                IDM_KEEP_INDEFINITE => {
                    state.power.activate_indefinite();
                    if let Some(tray) = &mut state.tray {
                        tray.update(&state.icons, &state.power);
                        tray.show_balloon(
                            &state.icons,
                            "Caffeine Active (Indefinite)",
                            "Screen will stay awake until you turn it off.",
                            true,
                        );
                    }
                }
                IDM_CUSTOM => {
                    let choice = show_custom_timer_dialog(hwnd, &state.icons);
                    match choice {
                        Some(CustomTimerChoice::DurationSeconds(secs)) => {
                            state.power.activate_timed_seconds(secs);
                            if let Some(tray) = &mut state.tray {
                                tray.update(&state.icons, &state.power);
                                let mins = (secs + 59) / 60;
                                tray.show_balloon(
                                    &state.icons,
                                    "Caffeine Active",
                                    &format!("Screen will stay awake for {} minutes.", mins),
                                    true,
                                );
                            }
                        }
                        Some(CustomTimerChoice::SpecificTime(h, m)) => {
                            state.power.activate_until_time(h, m);
                            if let Some(tray) = &mut state.tray {
                                tray.update(&state.icons, &state.power);
                                tray.show_balloon(
                                    &state.icons,
                                    "Caffeine Active",
                                    &format!("Screen will stay awake until {:02}:{:02}.", h, m),
                                    true,
                                );
                            }
                        }
                        None => {}
                    }
                }
                IDM_AUTOSTART => {
                    let current = service::is_autostart_enabled();
                    let _ = service::set_autostart(!current, None);
                    if let Some(tray) = &mut state.tray {
                        tray.show_balloon(
                            &state.icons,
                            "Startup Setting Changed",
                            if !current {
                                "Caffeine will now start automatically with Windows."
                            } else {
                                "Caffeine removed from Windows startup."
                            },
                            state.power.is_active(),
                        );
                    }
                }
                IDM_SETUP => {
                    let action = show_setup_dialog(hwnd, &state.icons);
                    match action {
                        SetupAction::Install {
                            autostart,
                            copy_to_appdata,
                        } => {
                            if copy_to_appdata {
                                let _ = service::install_self(autostart);
                            } else if autostart {
                                let _ = service::set_autostart(true, None);
                            }
                        }
                        SetupAction::Uninstall => {
                            let _ = service::uninstall_self();
                            unsafe {
                                PostQuitMessage(0);
                            }
                        }
                        _ => {}
                    }
                }
                IDM_ABOUT => {
                    show_message_box(
                        "About Caffeine",
                        "Caffeine for Windows\nVersion 1.0.0\n\n\
                         Ultra-lightweight display sleep inhibitor.\n\
                         Handcrafted Pixel Art & Pure Native Rust.\n\n\
                         • System tray quick presets: 15m, 30m, 60m, Indefinite\n\
                         • Custom timer: minutes, hours, or specific off time\n\
                         • Background service registration for Windows startup\n\n\
                         Keeps your PC awake on your terms.",
                    );
                }
                IDM_EXIT => {
                    unsafe {
                        PostQuitMessage(0);
                    }
                }
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn show_message_box(title: &str, message: &str) {
    let wide_title: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let wide_msg: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(
            null_mut(),
            wide_msg.as_ptr(),
            wide_title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
