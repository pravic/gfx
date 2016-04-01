// Copyright 2016 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{mem, ptr};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use kernel32;
use user32;
use winapi::*;

use sciter;
use sciter::types as sctypes;
use ::{_API};

pub fn init_sciter(hwnd: HWND, swap_chain: *mut IDXGISwapChain) -> Result<sciter::Host, ()>
{
  // create Sciter over DirectX Window
  let hwnd = hwnd as sctypes::HWINDOW;
  let ok = (_API.SciterCreateOnDirectXWindow)(hwnd, swap_chain as *mut sctypes::IDXGISwapChain);
  if ok == 0 {
    return Err(())
  }

  // attach Sciter host to it
  let host = sciter::Host::attach(hwnd);
  return Ok(host);
}

#[link(name="ole32")]
extern "system"
{
  fn OleInitialize(pv: LPCVOID) -> i32; // HRESULT
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
      // skip WM_CREATE to defer sciter window creation
      WM_CREATE => (),

      _ => {
        let mut handled = 0;
        let re = (_API.SciterProcND)(hwnd as sctypes::HWINDOW, msg, wp as sctypes::WPARAM, lp as sctypes::LPARAM, &mut handled);
        if handled != 0 {
          // println!("wndproc {:04X} eated", msg);
          return re as LRESULT;
        }
      }
    }
    match msg {
      // WM_PAINT => {0},
      _ => {
        // println!("wndproc {:04X}", msg);
        user32::DefWindowProcW(hwnd, msg, wp, lp)
      }
    }
}

pub fn create(name: &str, width: INT, height: INT) -> Result<HWND, ()> {
    let class_name = name.to_wide_null();
    let window_name = name.to_wide_null();
    let hwnd = unsafe {
        let hinst: HINSTANCE = kernel32::GetModuleHandleW(ptr::null());

        OleInitialize(::std::ptr::null());

        user32::RegisterClassW(&WNDCLASSW {
            style: CS_DBLCLKS | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbWndExtra: 0,
            hInstance: hinst,
            lpszClassName: class_name.as_ptr(),
            .. mem::zeroed()
        });

         user32::CreateWindowExW(
            WS_EX_ACCEPTFILES | WS_EX_APPWINDOW, // dwExStyle
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
            CW_USEDEFAULT, // x
            CW_USEDEFAULT, // y
            width,
            height,
            ptr::null_mut(),
            ptr::null_mut(),
            hinst,
            ptr::null_mut(),
        )
    };
    if hwnd != ptr::null_mut() {
        Ok(hwnd)
    }else {
        Err(())
    }
}

pub fn show(hwnd: HWND) -> Result<(INT, INT), ()> {
    let mut rc = RECT { left:0, right:0, top:0, bottom:0 };
    unsafe {
        user32::ShowWindow(hwnd, SW_SHOW);
        if user32::GetClientRect(hwnd, &mut rc) == TRUE {
            Ok((rc.right - rc.left, rc.bottom - rc.top))
        }else {
            Err(())
        }
    }
}

trait ToWide {
    fn to_wide(&self) -> Vec<u16>;
    fn to_wide_null(&self) -> Vec<u16>;
}

impl<T> ToWide for T where T: AsRef<OsStr> {
    fn to_wide(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn to_wide_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}
