// Extension of example win32 window from:
//   https://github.com/pachi/rust_winapi_examples/blob/master/src/bin/02_window.rs
// with C++ code translated to Rust from MSDN:
//   https://learn.microsoft.com/en-us/windows/win32/inputdev/using-raw-input

#![cfg(windows)]
use std::error::Error;
use std::ptr::null_mut;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::{GetModuleFileNameW, GetModuleHandleW};
use winapi::um::winuser::*;

use winapi::{
    um::{
        winuser::{RAWINPUTDEVICE, RIDEV_NOLEGACY, RegisterRawInputDevices, GetRawInputData}, 
        errhandlingapi::GetLastError,
    },
    shared::{
        windef::HWND,
        minwindef::{UINT, FALSE},
        hidusage::{HID_USAGE_PAGE_GENERIC, HID_USAGE_GENERIC_MOUSE, HID_USAGE_GENERIC_KEYBOARD}
    }
};

use std::process::exit;

// Get a win32 lpstr from a &str, converting u8 to u16 and appending '\0'
fn to_wstring(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// Handle leftbuttonclick
unsafe fn on_lbuttondown(hwnd: HWND) {
    let hinstance = GetModuleHandleW(null_mut());
    let mut name: Vec<u16> = Vec::with_capacity(MAX_PATH as usize);
    let read_len = GetModuleFileNameW(hinstance, name.as_mut_ptr(), MAX_PATH as u32);
    name.set_len(read_len as usize);
    // We could convert name to String using String::from_utf16_lossy(&name)
    MessageBoxW(
        hwnd,
        name.as_ptr(),
        to_wstring("This program is:").as_ptr(),
        MB_OK | MB_ICONINFORMATION,
    );
}

unsafe fn on_raw_input(lparam: LPARAM) {
    let lparam = lparam as HRAWINPUT;
    let mut dwSize: UINT = 0;
    if 0 != GetRawInputData(lparam, RID_INPUT, null_mut(), &mut dwSize, std::mem::size_of::<RAWINPUTHEADER>() as u32) {
        println!("on_raw_input::GetRawInputData:: should return zero because pData is null");
        return;
    }

    println!("First get OK");

    let mut lpb = vec![0 as BYTE; dwSize as usize];
    if dwSize != GetRawInputData(lparam, RID_INPUT, lpb.as_mut_ptr() as *mut winapi::ctypes::c_void, &mut dwSize, std::mem::size_of::<RAWINPUTHEADER>() as u32) {
        println!("GetRawInputData does not return correct size !\n");
    }

    println!("Second get OK");

    let raw = lpb.as_mut_ptr() as *mut winapi::ctypes::c_void as *mut RAWINPUT;

    let t = (*raw).header.dwType;
    println!("First data acces ok - {}", t);

    if (*raw).header.dwType == RIM_TYPEKEYBOARD
    {
        println!(" Kbd: make={} Flags:{} Reserved:{} ExtraInformation:{}, msg={} VK={} \n",
            (*raw).data.keyboard().MakeCode, 
            (*raw).data.keyboard().Flags, 
            (*raw).data.keyboard().Reserved, 
            (*raw).data.keyboard().ExtraInformation, 
            (*raw).data.keyboard().Message, 
            (*raw).data.keyboard().VKey);
    }
    else if (*raw).header.dwType == RIM_TYPEMOUSE
    {
        println!("Mouse: usFlags={} ulButtons={} usButtonFlags={} usButtonData={} ulRawButtons={} lLastX={} lLastY={} ulExtraInformation={}\r\n",
            (*raw).data.mouse().usFlags, 
            (*raw).data.mouse().ulRawButtons,
            (*raw).data.mouse().usButtonFlags, 
            (*raw).data.mouse().usButtonData, 
            (*raw).data.mouse().ulRawButtons, 
            (*raw).data.mouse().lLastX, 
            (*raw).data.mouse().lLastY, 
            (*raw).data.mouse().ulExtraInformation);
    } 

    return;
}

// Window procedure function to handle events
pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => {
            DestroyWindow(hwnd);
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        WM_LBUTTONDOWN => {
            on_lbuttondown(hwnd);
        }
        WM_INPUT => {
            println!("Raw input");
            on_raw_input(lparam);
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }
    return 0;
}

// Declare class and instantiate window
fn create_main_window(name: &str, title: &str) -> Result<HWND, Box<dyn Error>> {
    let name = to_wstring(name);
    let title = to_wstring(title);

    unsafe {
        // Get handle to the file used to create the calling process
        let hinstance = GetModuleHandleW(null_mut());

        // Create and register window class
        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance, // Handle to the instance that contains the window procedure for the class
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: COLOR_WINDOWFRAME as HBRUSH,
            lpszMenuName: null_mut(),
            lpszClassName: name.as_ptr(),
            hIconSm: LoadIconW(null_mut(), IDI_APPLICATION),
        };

        // Register window class
        if RegisterClassExW(&wnd_class) == 0 {
            MessageBoxW(
                null_mut(),
                to_wstring("Window Registration Failed!").as_ptr(),
                to_wstring("Error").as_ptr(),
                MB_ICONEXCLAMATION | MB_OK,
            );
            return Err("Window Registration Failed".into());
        };

        // Create a window based on registered class
        let handle = CreateWindowExW(
            0,                                // dwExStyle
            name.as_ptr(),                    // lpClassName
            title.as_ptr(),                   // lpWindowName
            WS_OVERLAPPEDWINDOW, // dwStyle
            CW_USEDEFAULT,                    // Int x
            CW_USEDEFAULT,                    // Int y
            CW_USEDEFAULT,                    // Int nWidth
            CW_USEDEFAULT,                    // Int nHeight
            null_mut(),                       // hWndParent
            null_mut(),                       // hMenu
            hinstance,                        // hInstance
            null_mut(),                       // lpParam
        );

        if handle.is_null() {
            MessageBoxW(
                null_mut(),
                to_wstring("Window Creation Failed!").as_ptr(),
                to_wstring("Error!").as_ptr(),
                MB_ICONEXCLAMATION | MB_OK,
            );
            return Err("Window Creation Failed!".into());
        }

        Ok(handle)
    }
}

// Message handling loop
fn run_message_loop(hwnd: HWND) -> WPARAM {
    unsafe {
        let mut msg: MSG = std::mem::uninitialized();
        loop {
            // Get message from message queue
            if GetMessageW(&mut msg, hwnd, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            } else {
                // Return on error (<0) or exit (=0) cases
                return msg.wParam;
            }
        }
    }
}

fn main() {
    let hwnd = create_main_window("my_window", "Example window creation")
        .expect("Window creation failed!");

    let rid = [
            RAWINPUTDEVICE{
                usUsagePage: HID_USAGE_PAGE_GENERIC,
                usUsage: HID_USAGE_GENERIC_MOUSE,
                dwFlags: RIDEV_NOLEGACY,
                hwndTarget: hwnd,
            },        
            RAWINPUTDEVICE{
                usUsagePage: HID_USAGE_PAGE_GENERIC,
                usUsage: HID_USAGE_GENERIC_KEYBOARD,
                dwFlags: RIDEV_NOLEGACY,
                hwndTarget: hwnd,
            },
        ];
    
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
            
            let result = RegisterRawInputDevices(rid.as_ptr(), rid.len() as UINT, std::mem::size_of::<RAWINPUTDEVICE>() as UINT);
            if result == FALSE {
                let error = GetLastError() as i32;
                println!("Error while registering raw input devices: {}", error);
                exit(error);
            }
        }
    println!("Run message loop");
    run_message_loop(hwnd);
}
