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
		UI::{Controls::*, Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
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

struct WindowState {
	paths: Vec<String>,
	icons: Vec<HICON>,
	font: HFONT,
	theme: Theme,
	is_move: bool,
	hover: bool,
}

fn main() -> Result<()> {
	let args = Args::parse(std::env::args().collect());

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
		.rev()
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

	// Create font (Segoe UI, 10pt)
	let font = create_font();

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
		let (width, height) = calculate_window_size(&paths, font);

		// Get cursor position
		let mut cursor_pos = POINT::default();
		GetCursorPos(&mut cursor_pos)?;

		let state = Box::new(WindowState {
			paths,
			icons,
			font,
			theme,
			is_move: args.is_move,
			hover: false,
		});

		let hwnd = CreateWindowExW(
			WINDOW_EX_STYLE(0),
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

fn create_font() -> HFONT {
	unsafe {
		CreateFontW(
			-13, // 10pt at 96 DPI
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

fn calculate_window_size(paths: &[String], font: HFONT) -> (i32, i32) {
	unsafe {
		let hdc = GetDC(None);
		let old_font = SelectObject(hdc, HGDIOBJ(font.0));

		let mut max_text_width: i32 = 0;
		for path in paths {
			let name = Path::new(path)
				.file_name()
				.and_then(|n| n.to_str())
				.unwrap_or(path);
			let wide: Vec<u16> = name.encode_utf16().collect();
			let mut size = SIZE::default();
			let _ = GetTextExtentPoint32W(hdc, &wide, &mut size);
			max_text_width = max_text_width.max(size.cx);
		}

		SelectObject(hdc, old_font);
		let _ = ReleaseDC(None, hdc);

		// Match C# layout: icon(24) + textWidth + table_padding(10) + extra(45)
		let item_width = 24 + max_text_width + 10 + 45;
		// Form padding: 10 left + 10 right
		let width = item_width + 20;

		let caption_height = GetSystemMetrics(SM_CYCAPTION);
		// Match C#: totalHeight + padding(10+10) + captionHeight + 20
		let height = (paths.len() as i32 * 40) + 20 + caption_height + 20;

		(width, height)
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
				// Drag on move with left button held (matches C# MouseMove handler)
				if wparam.0 & MK_LBUTTON.0 as usize != 0 {
					let effect = if state.is_move {
						DROPEFFECT_MOVE
					} else {
						DROPEFFECT_COPY
					};

					if let Ok(result) = start_drag_drop(&state.paths, effect) {
						if result == DROPEFFECT_MOVE || result == DROPEFFECT_COPY {
							unsafe { PostQuitMessage(0) };
						}
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
					unsafe { let _ = DeleteObject(HGDIOBJ(state.font.0)); };
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

	let mut y = 10;
	for (i, path) in state.paths.iter().enumerate() {
		// Draw icon (24x24, centered vertically in 40px row)
		if let Some(icon) = state.icons.get(i) {
			if !icon.is_invalid() {
				let _ = unsafe { DrawIconEx(hdc, 10, y + 8, *icon, 24, 24, 0, None, DI_NORMAL) };
			}
		}

		// Draw filename
		let name = Path::new(path)
			.file_name()
			.and_then(|n| n.to_str())
			.unwrap_or(path);

		let mut item_rect = RECT {
			left: 40,
			top: y,
			right: rect.right - 10,
			bottom: y + 40,
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

		y += 40;
	}

	// Restore old font
	unsafe { SelectObject(hdc, old_font) };

	unsafe {
		let _ = EndPaint(hwnd, &ps);
	};

	Ok(())
}
