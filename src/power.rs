use std::time::Instant;
use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaffeineMode {
    Inactive,
    Indefinite,
    Timed {
        end_instant: Instant,
        total_secs: u64,
        target_hour: u16,
        target_min: u16,
    },
}

pub struct PowerManager {
    mode: CaffeineMode,
}

impl PowerManager {
    pub fn new() -> Self {
        Self {
            mode: CaffeineMode::Inactive,
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.mode, CaffeineMode::Inactive)
    }

    pub fn mode(&self) -> CaffeineMode {
        self.mode
    }

    pub fn activate_indefinite(&mut self) {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
        }
        self.mode = CaffeineMode::Indefinite;
    }

    pub fn activate_timed_seconds(&mut self, seconds: u64) {
        if seconds == 0 {
            self.deactivate();
            return;
        }

        let now = Instant::now();
        let end_instant = now + std::time::Duration::from_secs(seconds);
        let (cur_h, cur_m) = get_local_time();

        let add_mins = (seconds + 59) / 60;
        let mut target_m = cur_m as u64 + add_mins;
        let target_h = (cur_h as u64 + target_m / 60) % 24;
        target_m %= 60;

        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
        }

        self.mode = CaffeineMode::Timed {
            end_instant,
            total_secs: seconds,
            target_hour: target_h as u16,
            target_min: target_m as u16,
        };
    }

    pub fn activate_until_time(&mut self, target_hour: u16, target_min: u16) {
        let (cur_h, cur_m) = get_local_time();
        let cur_minutes = (cur_h as i32) * 60 + (cur_m as i32);
        let target_minutes = (target_hour as i32) * 60 + (target_min as i32);

        let diff_minutes = if target_minutes > cur_minutes {
            target_minutes - cur_minutes
        } else {
            // Tomorrow
            (24 * 60 - cur_minutes) + target_minutes
        };

        let seconds = (diff_minutes.max(1) as u64) * 60;
        let now = Instant::now();
        let end_instant = now + std::time::Duration::from_secs(seconds);

        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
        }

        self.mode = CaffeineMode::Timed {
            end_instant,
            total_secs: seconds,
            target_hour,
            target_min,
        };
    }

    pub fn deactivate(&mut self) {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
        }
        self.mode = CaffeineMode::Inactive;
    }

    /// Ticks every second. Returns true if timer just expired.
    pub fn tick(&mut self) -> bool {
        if let CaffeineMode::Timed { end_instant, .. } = self.mode {
            if Instant::now() >= end_instant {
                self.deactivate();
                return true;
            }
        }
        false
    }

    pub fn remaining_seconds(&self) -> Option<u64> {
        match self.mode {
            CaffeineMode::Inactive => None,
            CaffeineMode::Indefinite => None,
            CaffeineMode::Timed { end_instant, .. } => {
                let now = Instant::now();
                if end_instant > now {
                    Some((end_instant - now).as_secs())
                } else {
                    Some(0)
                }
            }
        }
    }

    pub fn tooltip_text(&self) -> String {
        match self.mode {
            CaffeineMode::Inactive => "Caffeine: Inactive (Normal sleep)".to_string(),
            CaffeineMode::Indefinite => "Caffeine: Active (Until turned off)".to_string(),
            CaffeineMode::Timed {
                target_hour,
                target_min,
                ..
            } => {
                if let Some(secs) = self.remaining_seconds() {
                    let mins = secs / 60;
                    if mins > 0 {
                        format!(
                            "Caffeine: Active ({}m remaining - until {:02}:{:02})",
                            mins + 1,
                            target_hour,
                            target_min
                        )
                    } else {
                        format!("Caffeine: Active ({}s remaining)", secs)
                    }
                } else {
                    "Caffeine: Active".to_string()
                }
            }
        }
    }

    pub fn menu_header_text(&self) -> String {
        match self.mode {
            CaffeineMode::Inactive => "○ Caffeine: Inactive (Allow Sleep)".to_string(),
            CaffeineMode::Indefinite => "● Caffeine: Active (Indefinite)".to_string(),
            CaffeineMode::Timed {
                target_hour,
                target_min,
                ..
            } => {
                if let Some(secs) = self.remaining_seconds() {
                    let mins = secs / 60;
                    if mins > 0 {
                        format!(
                            "● Caffeine: Active ({}m left, until {:02}:{:02})",
                            mins + 1,
                            target_hour,
                            target_min
                        )
                    } else {
                        format!("● Caffeine: Active ({}s left)", secs)
                    }
                } else {
                    "● Caffeine: Active".to_string()
                }
            }
        }
    }
}

impl Drop for PowerManager {
    fn drop(&mut self) {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
        }
    }
}

pub fn get_local_time() -> (u16, u16) {
    let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
    unsafe {
        GetLocalTime(&mut st);
    }
    (st.wHour, st.wMinute)
}
