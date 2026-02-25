use windows::{Win32::System::Registry::*, core::*};

#[derive(Clone, Copy)]
pub struct Theme {
	pub background: u32,
	pub hover: u32,
	pub text: u32,
}

impl Theme {
	pub const LIGHT: Theme = Theme {
		background: 0x00FFFFFF, // White
		hover: 0x00D3D3D3,      // LightGray
		text: 0x00000000,       // Black
	};

	pub const DARK: Theme = Theme {
		background: 0x00000000, // Black
		hover: 0x00696969,      // DimGray
		text: 0x00FFFFFF,       // White
	};

	pub fn detect() -> Theme {
		unsafe {
			let mut key: HKEY = HKEY::default();
			let subkey = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");

			if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, Some(0), KEY_READ, &mut key).is_ok() {
				let mut value: u32 = 0;
				let mut size = std::mem::size_of::<u32>() as u32;
				let mut value_type = REG_NONE;

				let result = RegQueryValueExW(
					key,
					w!("AppsUseLightTheme"),
					None,
					Some(&mut value_type),
					Some(&mut value as *mut u32 as *mut u8),
					Some(&mut size),
				);

				let _ = RegCloseKey(key);

				if result.is_ok() && value == 1 {
					return Theme::LIGHT;
				}
			}

			Theme::DARK
		}
	}
}
