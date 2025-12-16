//! Platform-dependent windows support.

use capi::sctypes::*;

pub trait BaseWindow {
	fn create(&mut self, rc: RECT, flags: UINT, parent: HWINDOW) -> HWINDOW;

	fn get_hwnd(&self) -> HWINDOW;

	fn collapse(&self, hide: bool);
	fn expand(&self, maximize: bool);
	fn dismiss(&self);

	fn set_title(&mut self, title: &str);
	fn get_title(&self) -> String;

	fn run_app(&self);
	fn quit_app(&self);
}

#[cfg(windows)]
mod windows {

	use capi::scdef::*;
	use capi::sctypes::*;
	use _API;

	#[link(name = "user32")]
	extern "system" {
		fn ShowWindow(hwnd: HWINDOW, show: INT) -> BOOL;
		fn PostMessageW(hwnd: HWINDOW, msg: UINT, w: WPARAM, l: LPARAM) -> BOOL;
		fn SetWindowTextW(hwnd: HWINDOW, s: LPCWSTR) -> BOOL;
		fn GetWindowTextLengthW(hwnd: HWINDOW) -> INT;
		fn GetWindowTextW(hwnd: HWINDOW, s: LPWSTR, l: INT) -> INT;
		fn GetMessageW(msg: LPMSG, hwnd: HWINDOW, min: UINT, max: UINT) -> BOOL;
		fn DispatchMessageW(msg: LPMSG) -> LRESULT;
		fn TranslateMessage(msg: LPMSG) -> BOOL;
		fn PostQuitMessage(code: INT);
	}

	#[link(name = "ole32")]
	extern "system" {
		fn OleInitialize(pv: LPCVOID) -> i32; // HRESULT
	}

	pub struct OsWindow {
		hwnd: HWINDOW,
		flags: UINT,
	}

	impl OsWindow {
		pub fn new() -> OsWindow {
			OsWindow {
				hwnd: 0 as HWINDOW,
				flags: 0,
			}
		}

		pub fn from(hwnd: HWINDOW) -> OsWindow {
			OsWindow { hwnd: hwnd, flags: 0 }
		}

		fn init_app() {
			unsafe { OleInitialize(::std::ptr::null()) };
		}
	}

	impl super::BaseWindow for OsWindow {
		/// Get native window handle.
		fn get_hwnd(&self) -> HWINDOW {
			return self.hwnd;
		}

		/// Create a new native window.
		fn create(&mut self, rc: RECT, flags: UINT, parent: HWINDOW) -> HWINDOW {
			if (flags & SCITER_CREATE_WINDOW_FLAGS::SW_MAIN.bits()) != 0 {
				OsWindow::init_app();
			}

			self.flags = flags;

			#[cfg(not(feature = "windowless"))]
			{
				let cb = ::std::ptr::null();
				self.hwnd = (_API.SciterCreateWindow)(flags, &rc, cb, 0 as LPVOID, parent);
				if self.hwnd.is_null() {
					panic!("Failed to create window!");
				}
			}
			#[cfg(feature = "windowless")]
			{
				let _ = rc;
				let _ = parent;
				let _ = &(_API.SciterVersion);
			}

			return self.hwnd;
		}

		/// Minimize or hide window.
		fn collapse(&self, hide: bool) {
			let n: INT = if hide { 0 } else { 6 };
			unsafe { ShowWindow(self.hwnd, n) };
		}

		/// Show or maximize window.
		fn expand(&self, maximize: bool) {
			let n: INT = if maximize { 3 } else { 1 };
			unsafe { ShowWindow(self.hwnd, n) };
		}

		/// Close window.
		fn dismiss(&self) {
			unsafe { PostMessageW(self.hwnd, 0x0010, 0, 0) };
		}

		/// Set native window title.
		fn set_title(&mut self, title: &str) {
			let s = s2w!(title);
			unsafe { SetWindowTextW(self.hwnd, s.as_ptr()) };
		}

		/// Get native window title.
		fn get_title(&self) -> String {
			let n = unsafe { GetWindowTextLengthW(self.hwnd) + 1 };
			let mut title: Vec<u16> = Vec::new();
			title.resize(n as usize, 0);
			unsafe { GetWindowTextW(self.hwnd, title.as_mut_ptr(), n) };
			return ::utf::w2s(title.as_ptr());
		}

		/// Run the main app message loop until window been closed.
		fn run_app(&self) {
			let mut msg = MSG {
				hwnd: 0 as HWINDOW,
				message: 0,
				wParam: 0,
				lParam: 0,
				time: 0,
				pt: POINT { x: 0, y: 0 },
			};
			let pmsg: LPMSG = &mut msg;
			let null: HWINDOW = ::std::ptr::null_mut();
			unsafe {
				while GetMessageW(pmsg, null, 0, 0) != 0 {
					TranslateMessage(pmsg);
					DispatchMessageW(pmsg);
				}
			};
		}

		/// Post app quit message.
		fn quit_app(&self) {
			unsafe { PostQuitMessage(0) };
		}
	}
}

#[cfg(target_os = "linux")]
mod linux {
	use super::BaseWindow;
	use capi::scdef::*;
	use capi::sctypes::*;
	use _API;

	use std::ptr;


	pub struct OsWindow {
		hwnd: HWINDOW,
		flags: UINT,
	}

	impl OsWindow {
		pub fn new() -> OsWindow {
			OsWindow {
				hwnd: 0 as HWINDOW,
				flags: 0,
			}
		}

		pub fn from(hwnd: HWINDOW) -> OsWindow {
			OsWindow { hwnd: hwnd, flags: 0 }
		}

		fn init_app() {
			(_API.SciterExec)(SCITER_APP_CMD::SCITER_APP_INIT.bits(), 0, 0);
		}

		fn window(&self) -> HWINDOW {
			self.get_hwnd()
		}
	}

	impl super::BaseWindow for OsWindow {
		/// Get native window handle.
		fn get_hwnd(&self) -> HWINDOW {
			return self.hwnd;
		}

		/// Create a new native window.
		fn create(&mut self, rc: RECT, flags: UINT, parent: HWINDOW) -> HWINDOW {
			if (flags & SCITER_CREATE_WINDOW_FLAGS::SW_MAIN.bits()) != 0 {
				OsWindow::init_app();
			}
			self.flags = flags;

			#[cfg(not(feature = "windowless"))]
			{
				self.hwnd = (_API.SciterCreateWindow)(flags, &rc, ptr::null(), ptr::null_mut(), parent);
				if self.hwnd.is_null() {
					panic!("Failed to create window!");
				}
			}
			#[cfg(feature = "windowless")]
			{
				let _ = rc;
				let _ = parent;
				let _ = &(_API.SciterVersion);
			}
			return self.hwnd;
		}

		/// Minimize or hide window.
		fn collapse(&self, hide: bool) {
			unsafe {
				if hide {
					(_API.SciterWindowExec)(
						self.window(),
						SCITER_WINDOW_CMD::SCITER_WINDOW_SET_STATE.bits(),
						SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_HIDDEN.bits(),
						0,
					);
				} else {
					(_API.SciterWindowExec)(
						self.window(),
						SCITER_WINDOW_CMD::SCITER_WINDOW_SET_STATE.bits(),
						SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_MINIMIZED.bits(),
						0,
					);
				}
			};
		}

		/// Show or maximize window.
		fn expand(&self, maximize: bool) {
			let wnd = self.window();
			unsafe {
				if maximize {
					(_API.SciterWindowExec)(
						wnd,
						SCITER_WINDOW_CMD::SCITER_WINDOW_SET_STATE.bits(),
						SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_MAXIMIZED.bits(),
						0,
					);
				} else {
					(_API.SciterWindowExec)(
						wnd,
						SCITER_WINDOW_CMD::SCITER_WINDOW_SET_STATE.bits(),
						SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_SHOWN.bits(),
						0,
					);
				}
			};
		}

		/// Close window.
		fn dismiss(&self) {
			println!("linux::OsWindow::dismiss()");
			unsafe {
				(_API.SciterWindowExec)(
					self.window(),
					SCITER_WINDOW_CMD::SCITER_WINDOW_SET_STATE.bits(),
					SCITER_WINDOW_STATE::SCITER_WINDOW_STATE_CLOSED.bits(),
					0, // Set to FALSE for request_close behaviour
				);
			};
		}

		/// Set native window title.
		fn set_title(&mut self, title: &str) {
			unimplemented!();
		}

		/// Get native window title.
		fn get_title(&self) -> String {
			unimplemented!();
		}

		/// Run the main app message loop until window been closed.
		fn run_app(&self) {
			(_API.SciterExec)(SCITER_APP_CMD::SCITER_APP_LOOP.bits(), 0, 0);
		}

		/// Post app quit message.
		fn quit_app(&self) {
			(_API.SciterExec)(SCITER_APP_CMD::SCITER_APP_STOP.bits(), 0, 0);
		}
	}
}

#[cfg(target_os = "macos")]
mod macos {

	extern crate objc_foundation;

	use self::objc_foundation::{INSString, NSString};
	use objc::runtime::{Class, Object};

	/// Activation policies that control whether and how an app may be activated.
	#[repr(C)]
	#[allow(dead_code)]
	enum NSApplicationActivationPolicy {
		Regular = 0,
		Accessory,
		Prohibited,
	}

	// Note: Starting some OSX version (perhaps, 10.13),
	// the AppKit framework isn't loaded implicitly.
	#[link(name = "CoreFoundation", kind = "framework")]
	extern "C" {}

	#[link(name = "AppKit", kind = "framework")]
	extern "C" {}

	use super::BaseWindow;
	use capi::scdef::*;
	use capi::sctypes::*;
	use _API;

	pub struct OsWindow {
		hwnd: HWINDOW,
		flags: UINT,
	}

	impl OsWindow {
		pub fn new() -> OsWindow {
			OsWindow {
				hwnd: 0 as HWINDOW,
				flags: 0,
			}
		}

		pub fn from(hwnd: HWINDOW) -> OsWindow {
			OsWindow { hwnd: hwnd, flags: 0 }
		}

		fn get_app() -> *mut Object {
			let cls = Class::get("NSApplication").expect("`NSApplication` is not registered.");
			let obj = unsafe { msg_send!(cls, sharedApplication) };
			return obj;
		}

		fn init_app() {
			// By default, unbundled apps start with `NSApplicationActivationPolicyProhibited` (no dock, no menu).
			let app = OsWindow::get_app();
			let _: () = unsafe { msg_send!(app, setActivationPolicy:NSApplicationActivationPolicy::Regular) };
		}

		fn view(&self) -> *mut Object {
			let hwnd = self.get_hwnd();
			let hwnd = hwnd as *mut Object;
			return hwnd;
		}

		fn window(&self) -> *mut Object {
			let hwnd = self.view();
			let obj: *mut Object = unsafe { msg_send!(hwnd, window) };
			assert!(!obj.is_null());
			return obj;
		}
	}

	impl super::BaseWindow for OsWindow {
		/// Get native window handle.
		fn get_hwnd(&self) -> HWINDOW {
			return self.hwnd;
		}

		/// Create a new native window.
		fn create(&mut self, rc: RECT, flags: UINT, parent: HWINDOW) -> HWINDOW {
			if (flags & SCITER_CREATE_WINDOW_FLAGS::SW_MAIN.bits()) != 0 {
				OsWindow::init_app();
			}

			self.flags = flags;

			#[cfg(not(feature = "windowless"))]
			{
				let w = rc.right - rc.left;
				let h = rc.bottom - rc.top;
				let prc: *const RECT = if w > 0 && h > 0 { &rc } else { std::ptr::null() };

				let cb = std::ptr::null();
				self.hwnd = (_API.SciterCreateWindow)(flags, prc, 0 as LPVOID, 0 as LPVOID, 0 as LPVOID);
				if self.hwnd.is_null() {
					panic!("Failed to create window!");
				}
			}
			#[cfg(feature = "windowless")]
			{
				let _ = rc;
				let _ = parent;
				let _ = &(_API.SciterVersion);
			}
			return self.hwnd;
		}

		/// Minimize or hide window.
		fn collapse(&self, hide: bool) {
			let wnd = self.window();
			if hide {
				let _: () = unsafe { msg_send!(wnd, orderOut:0) };
			} else {
				let hwnd = self.view();
				let _: () = unsafe { msg_send!(wnd, performMiniaturize:hwnd) };
			}
		}

		/// Show or maximize window.
		fn expand(&self, maximize: bool) {
			let wnd = self.window();
			if (self.flags & SCITER_CREATE_WINDOW_FLAGS::SW_TITLEBAR.bits()) != 0 {
				let app = OsWindow::get_app();
				let _: () = unsafe { msg_send!(app, activateIgnoringOtherApps:true) };
			}
			unsafe {
				let _: () = msg_send!(wnd, makeKeyAndOrderFront:0);
				// msg_send!(wnd, orderFrontRegardless);
			}
			if maximize {
				let _: () = unsafe { msg_send!(wnd, performZoom:0) };
			}
		}

		/// Close window.
		fn dismiss(&self) {
			let wnd = self.window();
			let _: () = unsafe { msg_send!(wnd, close) };
		}

		/// Set native window title.
		fn set_title(&mut self, title: &str) {
			let s = NSString::from_str(title);
			let wnd = self.window();
			let _: () = unsafe { msg_send!(wnd, setTitle:s) };
		}

		/// Get native window title.
		fn get_title(&self) -> String {
			String::new()
		}

		/// Run the main app message loop until window been closed.
		fn run_app(&self) {
			let app = OsWindow::get_app();
			let _: () = unsafe { msg_send!(app, finishLaunching) };
			let _: () = unsafe { msg_send!(app, run) };
		}

		/// Post app quit message.
		fn quit_app(&self) {
			let app = OsWindow::get_app();
			let _: () = unsafe { msg_send!(app, terminate:app) };
		}
	}
}

#[cfg(windows)]
pub type OsWindow = windows::OsWindow;

#[cfg(target_os = "linux")]
pub type OsWindow = linux::OsWindow;

#[cfg(target_os = "macos")]
pub type OsWindow = macos::OsWindow;
