//! Windows session/power events, plus WTS checks before and after sensitive reads.
#[cfg(not(windows))]
pub fn desktop_available() -> bool {
    false
}

#[cfg(windows)]
pub fn desktop_available() -> bool {
    use windows_sys::Win32::System::RemoteDesktop::*;
    unsafe {
        let mut data = std::ptr::null_mut();
        let mut size = 0;
        if WTSQuerySessionInformationW(
            WTS_CURRENT_SERVER_HANDLE,
            WTS_CURRENT_SESSION,
            WTSSessionInfoEx,
            &mut data,
            &mut size,
        ) == 0
        {
            return false;
        }
        let available = if !data.is_null() && size as usize >= std::mem::size_of::<WTSINFOEXW>() {
            let info = &*(data as *const WTSINFOEXW);
            info.Level == 1
                && info.Data.WTSInfoExLevel1.SessionFlags == 1
                && info.Data.WTSInfoExLevel1.SessionState == WTSActive
        } else {
            false
        };
        WTSFreeMemory(data as _);
        available
    }
}

#[cfg(windows)]
pub fn start(sessions: std::sync::Weak<super::vault_session::VaultSessions>) {
    if let Some(s) = sessions.upgrade() {
        s.require_platform_monitor();
    }
    // The router starts closed. Only successful OS registration enables sessions.
    std::thread::spawn(move || unsafe { run(sessions) });
}

#[cfg(windows)]
unsafe fn run(sessions: std::sync::Weak<super::vault_session::VaultSessions>) {
    use windows_sys::{
        core::w,
        Win32::{
            System::{LibraryLoader::GetModuleHandleW, RemoteDesktop::*},
            UI::WindowsAndMessaging::*,
        },
    };
    let instance = GetModuleHandleW(std::ptr::null());
    let class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        lpszClassName: w!("AegisLifecycle"),
        ..std::mem::zeroed()
    };
    if RegisterClassW(&class) == 0 {
        return;
    }
    // A hidden top-level window receives power broadcasts; HWND_MESSAGE does not.
    let window = CreateWindowExW(
        0,
        class.lpszClassName,
        w!("AegisLifecycle"),
        0,
        0,
        0,
        0,
        0,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        instance,
        std::ptr::null(),
    );
    if window.is_null() {
        UnregisterClassW(class.lpszClassName, instance);
        return;
    }
    let context = Box::into_raw(Box::new(sessions));
    SetWindowLongPtrW(window, GWLP_USERDATA, context as isize);
    if WTSRegisterSessionNotification(window, NOTIFY_FOR_THIS_SESSION) != 0
        && SetTimer(window, 1, 1000, None) != 0
    {
        if let Some(s) = (*context).upgrade() {
            s.set_monitor_alive(true);
        }
        let mut message = std::mem::zeroed();
        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    if let Some(s) = (*context).upgrade() {
        s.set_monitor_alive(false);
    }
    KillTimer(window, 1);
    WTSUnRegisterSessionNotification(window);
    SetWindowLongPtrW(window, GWLP_USERDATA, 0);
    DestroyWindow(window);
    drop(Box::from_raw(context));
    UnregisterClassW(class.lpszClassName, instance);
}

#[cfg(windows)]
unsafe extern "system" fn window_proc(
    window: windows_sys::Win32::Foundation::HWND,
    message: u32,
    event: usize,
    param: isize,
) -> isize {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let context = GetWindowLongPtrW(window, GWLP_USERDATA)
        as *const std::sync::Weak<super::vault_session::VaultSessions>;
    if !context.is_null() {
        if let Some(sessions) = (*context).upgrade() {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match message {
                    WM_DESTROY => {
                        sessions.set_monitor_alive(false);
                        PostQuitMessage(0);
                    }
                    WM_WTSSESSION_CHANGE => sessions.system_lock(), // All transitions revoke; never auto-unlock.
                    WM_POWERBROADCAST if matches!(event, 4 | 6 | 7 | 18) => sessions.system_lock(),
                    WM_TIMER if !desktop_available() => sessions.system_lock(),
                    _ => {}
                }
            }));
            if result.is_err() {
                sessions.set_monitor_alive(false);
            }
        } else {
            PostQuitMessage(0);
        }
    }
    DefWindowProcW(window, message, event, param)
}

#[cfg(all(test, windows))]
mod tests {
    #[test]
    fn registered_window_revokes_sessions_on_power_and_lock_messages() {
        use std::{
            sync::Arc,
            time::{Duration, Instant},
        };
        use windows_sys::{
            core::w,
            Win32::{System::Threading::GetCurrentProcessId, UI::WindowsAndMessaging::*},
        };
        let sessions = Arc::new(super::super::vault_session::VaultSessions::new(Arc::new(
            crate::secrets::SecretManager::new(),
        )));
        super::start(Arc::downgrade(&sessions));
        let deadline = Instant::now() + Duration::from_secs(5);
        while !sessions.platform_monitor_active() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            sessions.platform_monitor_active(),
            "Windows notification registration failed"
        );
        // A noninteractive runner must remain closed; never bypass its actual OS state.
        if !super::desktop_available() {
            assert!(sessions
                .unlock("test.vault", "invalid", "password")
                .is_err());
            return;
        }
        let envelope =
            super::super::VaultEngine::encrypt("PRIVATE", "platform-test-password").unwrap();
        unsafe {
            let window = FindWindowW(w!("AegisLifecycle"), w!("AegisLifecycle"));
            assert!(!window.is_null());
            let mut process = 0;
            GetWindowThreadProcessId(window, &mut process);
            assert_eq!(
                process,
                GetCurrentProcessId(),
                "Never send test events to another application's window"
            );
            for (message, event) in [
                (WM_WTSSESSION_CHANGE, 7),
                (WM_POWERBROADCAST, 4),
                (WM_POWERBROADCAST, 18),
            ] {
                let token = sessions
                    .unlock("test.vault", &envelope, "platform-test-password")
                    .unwrap();
                SendMessageW(window, message, event, 0);
                assert!(sessions.read(&token, "test.vault", &envelope).is_err());
            }
            SendMessageW(window, WM_CLOSE, 0, 0);
            assert!(!sessions.platform_monitor_active());
            assert!(sessions
                .unlock("test.vault", &envelope, "platform-test-password")
                .is_err());
        }
    }
}
