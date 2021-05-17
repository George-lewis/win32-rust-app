#![cfg_attr(release, windows_subsystem = "windows")]

use winapi::{
    shared::windef::{
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, HWND, RECT,
        },
    um::{
        libloaderapi::GetModuleHandleW,
        wingdi::*,
        winuser::*
    },
};

use std::ptr::null_mut as null;

use std::mem;

struct Win32String {
    inner: Vec<u16>,
    len: usize
}

impl Win32String {
    fn ptr(&self) -> *const u16 {
        self.inner.as_ptr()
    }
}

fn win32_string_term(string: &str) -> Win32String {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    Win32String {
        inner: OsStr::new(string).encode_wide().chain(once(0)).collect(),
        len: string.chars().count(),
    }
}

fn win32_string(string: &str) -> Win32String {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    Win32String {
        inner: OsStr::new(string).encode_wide().collect(),
        len: string.chars().count(),
    }
}

fn main() {
    let code = realmain();
    std::process::exit(code);
}

fn realmain() -> i32 {
    if !cfg!(windows) {
        eprintln!("NOT A WINDOWS SYSTEM??");
        1
    } else {
        winmain()
    }
}

static mut counter: u8 = 0;
static mut gtext: String = String::new();

#[cfg(windows)]
unsafe extern "system" fn window_proc(
    handle: HWND,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    match msg {
        WM_DESTROY | WM_QUIT => {
            PostQuitMessage(0);
            0
        }
        WM_DPICHANGED => {
            // SetWindowPos(handle, null())
            // AdjustWindowRectExForDpi()
            println!("STUB: DPI CHANGE!");
            0
        }
        WM_LBUTTONDOWN => {
            
            println!("CLICK!");
            counter += 1;
            InvalidateRect(handle, null(), 1);
            0
        }
        WM_CHAR => {
            // Not ideal
            // Breaks if the char is multi-unit
            let c = std::char::from_u32_unchecked(wparam as _);

            match c as u8 {
                // Backspace
                8 => {
                    gtext.pop();
                },
                // Tab
                9 => {
                    gtext.push_str("    ");
                }
                _ => {
                    gtext.push(c);
                }
            };
            InvalidateRect(handle, null(), 1);
            0
        }
        WM_PAINT => {
            // Window rect
            let mut rect: RECT = mem::zeroed();
            GetClientRect(handle, &mut rect);

            let mut ps: PAINTSTRUCT = mem::zeroed();

            let hdc = BeginPaint(handle, &mut ps);

            // Fill it with the window color
            FillRect(hdc, &ps.rcPaint, (1 + COLOR_WINDOW) as *mut _);

            let text = win32_string(&format!("{}|", &gtext));

            let dpi = GetDpiForWindow(handle);

            let font_family = win32_string_term("Comic Sans MS");

            // Adjust for the same size on each monitor
            let height = (0.25*dpi as f64) as i32;
            // let height = (30*96)/dpi as i32;

            let font = CreateFontW(
                height,
                0,
                0,
                0,
                FW_DONTCARE,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                DEFAULT_PITCH | FF_DONTCARE,
                font_family.ptr(),
            );

            SelectObject(hdc, font as _);

            if text.len > 0 {
                DrawTextW(hdc, text.ptr(), text.len as _, &mut rect, DT_LEFT | DT_TOP);
            }

            EndPaint(handle, &ps);
            0
        }
        WM_GETMINMAXINFO => {
            let info = lparam as LPMINMAXINFO;
            (*info).ptMinTrackSize.x = 500;
            (*info).ptMinTrackSize.y = 300;
            0
        }
        _ => DefWindowProcW(handle, msg, wparam, lparam),
    }
}

#[cfg(windows)]
fn winmain() -> i32 {
    unsafe {
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let class_name = win32_string_term("class-name");
    let window_name = win32_string_term("The window");

    let instance = unsafe { GetModuleHandleW(null()) };

    let wnd_class = WNDCLASSW {
        style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        lpszClassName: class_name.ptr(),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hIcon: null(),
        hCursor: null(),
        hbrBackground: null(),
        lpszMenuName: null(),
    };

    unsafe {
        RegisterClassW(&wnd_class as *const WNDCLASSW);

        let handle = CreateWindowExW(
            0,
            class_name.ptr(),
            window_name.ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            500,
            300,
            null(),
            null(),
            null(),
            null(),
        );
        ShowWindow(handle, SW_SHOWNORMAL);

        // Button

        // let btn = CreateWindowExW(
        //     0,
        //     win32_string_term("BUTTON").ptr(),
        //     win32_string_term("BRUH").ptr(),
        //     WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON,
        //     10,
        //     10,
        //     100,
        //     25,
        //     handle,
        //     null(),
        //     GetWindowLongPtrW(handle, GWLP_HINSTANCE) as _,
        //     null()
        // );
        
        let mut first: u8 = 2;
        loop {
            let mut msg: MSG = mem::zeroed();
            let result = GetMessageW(&mut msg as *mut MSG, null(), 0, 0);
            if result <= 0 {
                return result;
            }
            TranslateMessage(&msg as *const MSG);
            DispatchMessageW(&msg as *const MSG);
            if first > 0 {
                // Have to do this multiple times for some reason
                InvalidateRect(handle, null(), 1);
                first -= 1;
            }
        }
    }
}
