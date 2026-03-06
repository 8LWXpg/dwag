use std::path::Path;

use windows::{
	Win32::{
		Foundation::*,
		Graphics::{Dwm::*, Gdi::*},
		Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
		System::{
			LibraryLoader::GetModuleHandleW,
			Ole::{DROPEFFECT_COPY, DROPEFFECT_MOVE},
			SystemServices::MK_LBUTTON,
		},
		UI::{Controls::*, HiDpi::*, Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
	},
	core::*,
};

mod args;
mod drag_drop;
mod theme;

use args::Args;
use drag_drop::start_drag_drop;
use theme::Theme;

const CLASS_NAME: PCWSTR = w!("DwagWindow");
const BASE_DPI: i32 = 96;

struct WindowState {
	paths: Vec<String>,
	icons: Vec<HICON>,
	font: HFONT,
	theme: Theme,
	is_move: bool,
	hover: bool,
	layout: Layout,
}

/// Scale a pixel value from BASE_DPI DPI baseline to the actual DPI.
fn scale(px: i32, dpi: i32) -> i32 {
	px * dpi / BASE_DPI
}

/// Extract the filename portion of a path for display.
fn display_name(path: &str) -> &str {
	Path::new(path)
		.file_name()
		.and_then(|n| n.to_str())
		.unwrap_or(path)
}

/// All layout dimensions for the file list window, pre-scaled to the target DPI
///
/// Base values (at BASE_DPI) are defined as constants; [`Layout::new`] scales them
/// once so every consumer works with pixel-ready values
struct Layout {
	/// The DPI this layout was computed for.
	dpi: u32,
	/// File icon width and height (24 @ BASE_DPI)
	icon_size: i32,
	/// Height of each file entry row (40 @ BASE_DPI)
	row_height: i32,
	/// Content inset from the window client-area edges
	/// left, right, top, and bottom (5 @ BASE_DPI)
	padding: i32,
	/// Horizontal gap between the icon and the filename text (6 @ BASE_DPI)
	icon_text_gap: i32,
}

impl Layout {
	fn new(dpi: i32) -> Self {
		Self {
			dpi: dpi as u32,
			icon_size: scale(24, dpi),
			row_height: scale(30, dpi),
			padding: scale(10, dpi),
			icon_text_gap: scale(5, dpi),
		}
	}

	/// X coordinate where filename text begins.
	fn text_left(&self) -> i32 {
		self.padding + self.icon_size + self.icon_text_gap
	}

	/// Calculate window dimensions for the given max text width and item count
	/// Returns the outer window size (including title bar and borders)
	fn window_size(&self, max_text_width: i32, item_count: i32) -> (i32, i32) {
		let client_w =
			self.padding + self.icon_size + self.icon_text_gap + max_text_width + self.padding;
		let client_h = self.padding + item_count * self.row_height + self.padding;

		let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU;
		let mut rc = RECT {
			left: 0,
			top: 0,
			right: client_w,
			bottom: client_h,
		};
		unsafe {
			let _ = AdjustWindowRectExForDpi(&mut rc, style, false, WS_EX_DLGMODALFRAME, self.dpi);
		}

		(rc.right - rc.left, rc.bottom - rc.top)
	}
}

fn main() -> Result<()> {
	let args = Args::parse(std::env::args().collect());

	unsafe {
		let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
	}

	if args.help || args.files.is_empty() {
		let help = args.get_help();
		unsafe {
			MessageBoxW(None, &HSTRING::from(&help), w!("dwag"), MB_OK);
		}
		return Ok(());
	}

	// Resolve paths: reverse order, combine with CWD, filter existing
	let paths: Vec<String> = args
		.files
		.into_iter()
		.filter_map(|p| {
			std::path::absolute(&p)
				.ok()
				.and_then(|abs| abs.to_str().map(String::from))
		})
		.filter(|p| Path::new(p).exists())
		.collect();

	if paths.is_empty() {
		unsafe {
			MessageBoxW(
				None,
				w!("Files/folders do not exist"),
				w!("dwag"),
				MB_OK | MB_ICONERROR,
			);
		}
		return Ok(());
	}

	// Extract shell icons for each path
	let icons: Vec<HICON> = paths.iter().map(|p| extract_icon(p)).collect();

	// Get system DPI for scaling
	let dpi = unsafe {
		let hdc = GetDC(None);
		let dpi = GetDeviceCaps(Some(hdc), LOGPIXELSY);
		let _ = ReleaseDC(None, hdc);
		dpi
	};

	// Create font (Segoe UI, 10pt)
	let font = create_font(dpi);

	let layout = Layout::new(dpi);
	let theme = Theme::detect();

	unsafe {
		let instance = GetModuleHandleW(None)?;

		let wc = WNDCLASSW {
			lpfnWndProc: Some(wndproc),
			hInstance: instance.into(),
			lpszClassName: CLASS_NAME,
			hCursor: LoadCursorW(None, IDC_HAND)?,
			hbrBackground: CreateSolidBrush(COLORREF(theme.background)),
			..Default::default()
		};

		let class_registered = RegisterClassW(&wc);

		assert!(class_registered != 0);

		// Calculate window size by measuring text
		let max_text_width = measure_max_text_width(&paths, font);
		let (width, height) = layout.window_size(max_text_width, paths.len() as i32);

		// Get cursor position
		let mut cursor_pos = POINT::default();
		GetCursorPos(&mut cursor_pos)?;

		let state = Box::new(WindowState {
			paths,
			icons,
			font,
			theme,
			is_move: args.r#move,
			hover: false,
			layout,
		});

		let hwnd = CreateWindowExW(
			WS_EX_DLGMODALFRAME,
			CLASS_NAME,
			w!("dwag"),
			WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
			cursor_pos.x,
			cursor_pos.y,
			width,
			height,
			None,
			None,
			Some(instance.into()),
			Some(Box::into_raw(state) as *const _),
		)?;

		// Enable dark mode
		let dark_mode: i32 = 1;
		DwmSetWindowAttribute(
			hwnd,
			DWMWA_USE_IMMERSIVE_DARK_MODE,
			&dark_mode as *const _ as *const _,
			std::mem::size_of::<i32>() as u32,
		)?;

		let _ = ShowWindow(hwnd, SW_SHOW);
		SetWindowPos(
			hwnd,
			Some(HWND_TOPMOST),
			0,
			0,
			0,
			0,
			SWP_NOMOVE | SWP_NOSIZE,
		)?;

		let mut msg = MSG::default();
		while GetMessageW(&mut msg, None, 0, 0).into() {
			let _ = TranslateMessage(&msg);
			DispatchMessageW(&msg);
		}

		Ok(())
	}
}

fn extract_icon(path: &str) -> HICON {
	unsafe {
		let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
		let mut shfi = SHFILEINFOW::default();
		let result = SHGetFileInfoW(
			PCWSTR(wide_path.as_ptr()),
			FILE_FLAGS_AND_ATTRIBUTES(0),
			Some(&mut shfi),
			std::mem::size_of::<SHFILEINFOW>() as u32,
			SHGFI_ICON,
		);
		if result != 0 && !shfi.hIcon.is_invalid() {
			shfi.hIcon
		} else {
			HICON::default()
		}
	}
}

fn create_font(dpi: i32) -> HFONT {
	unsafe {
		// GDI+ computes lfHeight = -(emSize * dpiY / 72) for GraphicsUnit.Point
		let height = -(10 * dpi / 72);

		CreateFontW(
			height,
			0,
			0,
			0,
			FW_NORMAL.0 as i32,
			0,
			0,
			0,
			DEFAULT_CHARSET,
			OUT_DEFAULT_PRECIS,
			CLIP_DEFAULT_PRECIS,
			CLEARTYPE_QUALITY,
			DEFAULT_PITCH.0 as u32,
			w!("Segoe UI"),
		)
	}
}

/// Measure the widest filename among `paths` using the given `font`.
fn measure_max_text_width(paths: &[String], font: HFONT) -> i32 {
	unsafe {
		let hdc = GetDC(None);
		let old_font = SelectObject(hdc, HGDIOBJ(font.0));

		let mut max_width: i32 = 0;
		for path in paths {
			let name = display_name(path);
			let wide: Vec<u16> = name.encode_utf16().collect();
			let mut size = SIZE::default();
			let _ = GetTextExtentPoint32W(hdc, &wide, &mut size);
			max_width = max_width.max(size.cx);
		}

		SelectObject(hdc, old_font);
		let _ = ReleaseDC(None, hdc);
		max_width
	}
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
	match msg {
		WM_CREATE => unsafe {
			let create_struct = lparam.0 as *const CREATESTRUCTW;
			let state = (*create_struct).lpCreateParams as *mut WindowState;
			SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
			LRESULT(0)
		},
		WM_PAINT => {
			let state = get_state(hwnd);
			if let Some(state) = state {
				paint_window(hwnd, state).unwrap();
			}
			LRESULT(0)
		}
		WM_MOUSEMOVE => {
			let state = get_state_mut(hwnd);
			if let Some(state) = state {
				// Drag on move with left button held
				if wparam.0 & MK_LBUTTON.0 as usize != 0 {
					let effect = if state.is_move {
						DROPEFFECT_MOVE
					} else {
						DROPEFFECT_COPY
					};

					if let Ok(result) = start_drag_drop(&state.paths, effect)
						&& (result == DROPEFFECT_MOVE || result == DROPEFFECT_COPY)
					{
						unsafe { PostQuitMessage(0) };
					}
				} else if !state.hover {
					state.hover = true;
					unsafe {
						SetClassLongPtrW(
							hwnd,
							GCLP_HBRBACKGROUND,
							CreateSolidBrush(COLORREF(state.theme.hover)).0 as isize,
						);
						let _ = InvalidateRect(Some(hwnd), None, true);
					}

					// Track mouse leave
					let mut tme = TRACKMOUSEEVENT {
						cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
						dwFlags: TME_LEAVE,
						hwndTrack: hwnd,
						dwHoverTime: 0,
					};
					unsafe {
						let _ = TrackMouseEvent(&mut tme);
					};
				}
			}
			LRESULT(0)
		}
		WM_MOUSELEAVE => {
			let state = get_state_mut(hwnd);
			if let Some(state) = state {
				state.hover = false;
				unsafe {
					SetClassLongPtrW(
						hwnd,
						GCLP_HBRBACKGROUND,
						CreateSolidBrush(COLORREF(state.theme.background)).0 as isize,
					);
					let _ = InvalidateRect(Some(hwnd), None, true);
				};
			}
			LRESULT(0)
		}
		WM_DPICHANGED => {
			let state = get_state_mut(hwnd);
			if let Some(state) = state {
				let new_dpi = (wparam.0 & 0xFFFF) as i32;

				// Recreate the font for the new DPI
				if !state.font.is_invalid() {
					unsafe {
						let _ = DeleteObject(HGDIOBJ(state.font.0));
					};
				}
				state.font = create_font(new_dpi);

				// Recalculate layout dimensions
				state.layout = Layout::new(new_dpi);

				// Remeasure text and resize window
				let max_text_width = measure_max_text_width(&state.paths, state.font);
				let (width, height) = state
					.layout
					.window_size(max_text_width, state.paths.len() as i32);

				// Use the suggested rect's position, but our calculated size
				let suggested = unsafe { &*(lparam.0 as *const RECT) };
				unsafe {
					let _ = SetWindowPos(
						hwnd,
						None,
						suggested.left,
						suggested.top,
						width,
						height,
						SWP_NOZORDER | SWP_NOACTIVATE,
					);
					let _ = InvalidateRect(Some(hwnd), None, true);
				}
			}
			LRESULT(0)
		}
		WM_DESTROY => {
			let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
			if !state_ptr.is_null() {
				let state = unsafe { Box::from_raw(state_ptr) };
				for icon in &state.icons {
					if !icon.is_invalid() {
						let _ = unsafe { DestroyIcon(*icon) };
					}
				}
				if !state.font.is_invalid() {
					unsafe {
						let _ = DeleteObject(HGDIOBJ(state.font.0));
					};
				}
			}
			unsafe { PostQuitMessage(0) };
			LRESULT(0)
		}
		_ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
	}
}

fn get_state<'a>(hwnd: HWND) -> Option<&'a WindowState> {
	let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const WindowState;
	if ptr.is_null() {
		None
	} else {
		Some(unsafe { &*ptr })
	}
}

fn get_state_mut<'a>(hwnd: HWND) -> Option<&'a mut WindowState> {
	let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut WindowState;
	if ptr.is_null() {
		None
	} else {
		Some(unsafe { &mut *ptr })
	}
}

fn paint_window(hwnd: HWND, state: &WindowState) -> Result<()> {
	let mut ps = PAINTSTRUCT::default();
	let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

	let mut rect = RECT::default();
	(unsafe { GetClientRect(hwnd, &mut rect) })?;

	unsafe {
		SetTextColor(hdc, COLORREF(state.theme.text));
		SetBkMode(hdc, TRANSPARENT);
	}

	// Select custom font
	let old_font = unsafe { SelectObject(hdc, HGDIOBJ(state.font.0)) };

	let layout = &state.layout;

	let mut y = layout.padding;
	for (i, path) in state.paths.iter().enumerate() {
		// Draw icon (scaled, centered vertically in row)
		if let Some(icon) = state.icons.get(i)
			&& !icon.is_invalid()
		{
			let icon_y = y + (layout.row_height - layout.icon_size) / 2;
			let _ = unsafe {
				DrawIconEx(
					hdc,
					layout.padding,
					icon_y,
					*icon,
					layout.icon_size,
					layout.icon_size,
					0,
					None,
					DI_NORMAL,
				)
			};
		}

		// Draw filename
		let name = display_name(path);

		let mut item_rect = RECT {
			left: layout.text_left(),
			top: y,
			right: rect.right - layout.padding,
			bottom: y + layout.row_height,
		};

		let mut text = name.encode_utf16().collect::<Vec<u16>>();

		unsafe {
			DrawTextW(
				hdc,
				&mut text,
				&mut item_rect,
				DT_LEFT | DT_VCENTER | DT_SINGLELINE,
			)
		};

		y += layout.row_height;
	}

	// Restore old font
	unsafe { SelectObject(hdc, old_font) };

	unsafe {
		let _ = EndPaint(hwnd, &ps);
	};

	Ok(())
}
