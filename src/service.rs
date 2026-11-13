use std::path::{Path, PathBuf};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_KEY_PATH: &[u16] = &[
    b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16, b'r' as u16,
    b'e' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16,
    b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
    b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16, b'C' as u16,
    b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16,
    b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'\\' as u16,
    b'R' as u16, b'u' as u16, b'n' as u16, 0,
];

const APP_NAME_W: &[u16] = &[
    b'C' as u16, b'a' as u16, b'f' as u16, b'f' as u16, b'e' as u16, b'i' as u16, b'n' as u16,
    b'e' as u16, 0,
];

pub fn get_installed_exe_path() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join("Caffeine").join("caffeine.exe")
    } else {
        PathBuf::from(r"C:\Program Files\Caffeine\caffeine.exe")
    }
}

pub fn is_currently_installed() -> bool {
    if let Ok(current) = std::env::current_exe() {
        let installed = get_installed_exe_path();
        current.to_string_lossy().eq_ignore_ascii_case(&installed.to_string_lossy())
    } else {
        false
    }
}

pub fn is_autostart_enabled() -> bool {
    let mut hkey: HKEY = std::ptr::null_mut();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            RUN_KEY_PATH.as_ptr(),
            0,
            KEY_READ,
            &mut hkey,
        )
    };

    if status != 0 || hkey.is_null() {
        return false;
    }

    let mut buf = [0u16; 512];
    let mut size = (buf.len() * 2) as u32;
    let mut val_type: u32 = 0;

    let res = unsafe {
        RegQueryValueExW(
            hkey,
            APP_NAME_W.as_ptr(),
            std::ptr::null_mut(),
            &mut val_type,
            buf.as_mut_ptr() as *mut u8,
            &mut size,
        )
    };

    unsafe {
        RegCloseKey(hkey);
    }

    res == 0
}

pub fn set_autostart(enable: bool, exe_path: Option<&Path>) -> Result<(), String> {
    let mut hkey: HKEY = std::ptr::null_mut();

    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            RUN_KEY_PATH.as_ptr(),
            0,
            std::ptr::null_mut(),
            0,
            KEY_WRITE,
            std::ptr::null_mut(),
            &mut hkey,
            std::ptr::null_mut(),
        )
    };

    if status != 0 || hkey.is_null() {
        return Err(format!("Failed to open Run registry key: code {}", status));
    }

    let res = if enable {
        let path = match exe_path {
            Some(p) => p.to_path_buf(),
            None => std::env::current_exe().unwrap_or_else(|_| get_installed_exe_path()),
        };
        
        let cmd = format!("\"{}\" --tray", path.display());
        let wide: Vec<u16> = cmd.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            RegSetValueExW(
                hkey,
                APP_NAME_W.as_ptr(),
                0,
                REG_SZ,
                wide.as_ptr() as *const u8,
                (wide.len() * 2) as u32,
            )
        }
    } else {
        unsafe { RegDeleteValueW(hkey, APP_NAME_W.as_ptr()) }
    };

    unsafe {
        RegCloseKey(hkey);
    }

    if res == 0 || (!enable && res == 2) {
        // 2 = ERROR_FILE_NOT_FOUND, fine when deleting non-existent key
        Ok(())
    } else {
        Err(format!("Registry operation failed with code: {}", res))
    }
}

pub fn install_self(enable_autostart: bool) -> Result<PathBuf, String> {
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let target_path = get_installed_exe_path();

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // If current exe is not already target, copy it
    if !current_exe.to_string_lossy().eq_ignore_ascii_case(&target_path.to_string_lossy()) {
        std::fs::copy(&current_exe, &target_path).map_err(|e| {
            format!(
                "Failed to copy {} to {}: {}",
                current_exe.display(),
                target_path.display(),
                e
            )
        })?;
    }

    if enable_autostart {
        set_autostart(true, Some(&target_path))?;
    }

    Ok(target_path)
}

pub fn uninstall_self() -> Result<(), String> {
    set_autostart(false, None)?;

    let target_path = get_installed_exe_path();
    if target_path.exists() {
        // Can try removing file or mark for delete
        let _ = std::fs::remove_file(&target_path);
        if let Some(parent) = target_path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }

    Ok(())
}
