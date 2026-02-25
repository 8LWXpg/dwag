use std::cell::Cell;
use std::mem::ManuallyDrop;
use windows::{
	Win32::{
		Foundation::*,
		System::{Com::*, Memory::*, Ole::*, SystemServices::*},
		UI::Shell::*,
	},
	core::*,
};

#[derive(Clone)]
#[implement(IDropSource)]
struct DropSource;

impl IDropSource_Impl for DropSource_Impl {
	fn QueryContinueDrag(&self, fescapepressed: BOOL, grfkeystate: MODIFIERKEYS_FLAGS) -> HRESULT {
		// If the <Escape> key has been pressed since the last call, cancel the drop
		if fescapepressed.as_bool() {
			return DRAGDROP_S_CANCEL;
		}

		// If the <LeftMouse> button has been released, do the drop
		if grfkeystate.0 & MK_LBUTTON.0 == 0 {
			return DRAGDROP_S_DROP;
		}

		// Continue with the drag-drop
		S_OK
	}

	fn GiveFeedback(&self, _dweffect: DROPEFFECT) -> HRESULT {
		DRAGDROP_S_USEDEFAULTCURSORS
	}
}

#[derive(Clone)]
#[implement(IDataObject)]
struct DataObject {
	paths: Vec<String>,
}

impl DataObject {
	fn new(paths: Vec<String>) -> Self {
		Self { paths }
	}

	/// The single FORMATETC we advertise: CF_HDROP via HGLOBAL.
	fn supported_format() -> FORMATETC {
		FORMATETC {
			cfFormat: CF_HDROP.0,
			ptd: std::ptr::null_mut(),
			dwAspect: DVASPECT_CONTENT.0,
			lindex: -1,
			tymed: TYMED_HGLOBAL.0 as u32,
		}
	}

	/// Match a requested FORMATETC against our supported format (C++ LookupFormatEtc).
	/// Uses bitwise AND for tymed (it is a bitmask per COM spec).
	fn matches_format(requested: &FORMATETC) -> bool {
		let supported = Self::supported_format();
		requested.cfFormat == supported.cfFormat
			&& requested.dwAspect == supported.dwAspect
			&& (requested.tymed & supported.tymed) != 0
	}

	/// Build a DROPFILES HGLOBAL containing all paths as wide strings
	fn build_dropfiles_hglobal(&self) -> Result<HGLOBAL> {
		unsafe {
			// Collect all paths as null-terminated wide strings
			let mut wide_paths: Vec<Vec<u16>> = Vec::new();
			let mut total_wide_chars: usize = 0;

			for path in &self.paths {
				let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
				total_wide_chars += wide.len();
				wide_paths.push(wide);
			}

			// DROPFILES header + null-terminated wide strings + final null terminator
			let buf_size = size_of::<DROPFILES>() + size_of::<u16>() * (total_wide_chars + 1);

			let hglobal = GlobalAlloc(GMEM_MOVEABLE, buf_size)?;
			let ptr = GlobalLock(hglobal) as *mut u8;

			if ptr.is_null() {
				GlobalFree(Some(hglobal))?;
				return Err(E_OUTOFMEMORY.into());
			}

			// Fill DROPFILES header
			let dropfiles = ptr as *mut DROPFILES;
			(*dropfiles).pFiles = size_of::<DROPFILES>() as u32;
			(*dropfiles).fWide = TRUE;

			// Copy file paths after the DROPFILES struct
			let mut offset = size_of::<DROPFILES>();
			for wide_path in &wide_paths {
				let dest = ptr.add(offset) as *mut u16;
				std::ptr::copy_nonoverlapping(wide_path.as_ptr(), dest, wide_path.len());
				offset += wide_path.len() * size_of::<u16>();
			}

			// Final null terminator
			*(ptr.add(offset) as *mut u16) = 0;

			let _ = GlobalUnlock(hglobal);
			Ok(hglobal)
		}
	}
}

impl IDataObject_Impl for DataObject_Impl {
	fn GetData(&self, pformatetc: *const FORMATETC) -> Result<STGMEDIUM> {
		unsafe {
			let format = &*pformatetc;

			// Try to match the requested FORMATETC with our supported format
			if !DataObject::matches_format(format) {
				return Err(DV_E_FORMATETC.into());
			}

			// Build the DROPFILES HGLOBAL with all file paths
			let hglobal = self.build_dropfiles_hglobal()?;

			// Transfer data into the supplied storage-medium
			let mut medium = STGMEDIUM {
				tymed: TYMED_HGLOBAL.0 as u32,
				u: Default::default(),
				pUnkForRelease: ManuallyDrop::new(None),
			};
			medium.u.hGlobal = hglobal;

			Ok(medium)
		}
	}

	fn GetDataHere(&self, _pformatetc: *const FORMATETC, _pmedium: *mut STGMEDIUM) -> Result<()> {
		// GetDataHere is only required for IStream and IStorage mediums;
		// it is an error to call it for HGLOBAL and other clipboard formats.
		Err(DV_E_FORMATETC.into())
	}

	fn QueryGetData(&self, pformatetc: *const FORMATETC) -> HRESULT {
		unsafe {
			let format = &*pformatetc;
			if DataObject::matches_format(format) {
				S_OK
			} else {
				DV_E_FORMATETC
			}
		}
	}

	fn GetCanonicalFormatEtc(
		&self,
		_pformatetcin: *const FORMATETC,
		pformatetcout: *mut FORMATETC,
	) -> HRESULT {
		// Must set ptd to NULL even though we don't do anything else
		unsafe {
			(*pformatetcout).ptd = std::ptr::null_mut();
		}
		E_NOTIMPL
	}

	fn SetData(
		&self,
		_pformatetc: *const FORMATETC,
		_pmedium: *const STGMEDIUM,
		_frelease: BOOL,
	) -> Result<()> {
		Err(E_NOTIMPL.into())
	}

	fn EnumFormatEtc(&self, dwdirection: u32) -> Result<IEnumFORMATETC> {
		if dwdirection == DATADIR_GET.0 as u32 {
			let formats = [DataObject::supported_format()];
			let enumerator = EnumFormatEtc::new(&formats);
			Ok(enumerator.into())
		} else {
			Err(E_NOTIMPL.into())
		}
	}

	fn DAdvise(
		&self,
		_pformatetc: *const FORMATETC,
		_advf: u32,
		_padvsink: Ref<IAdviseSink>,
	) -> Result<u32> {
		Err(OLE_E_ADVISENOTSUPPORTED.into())
	}

	fn DUnadvise(&self, _dwconnection: u32) -> Result<()> {
		Err(OLE_E_ADVISENOTSUPPORTED.into())
	}

	fn EnumDAdvise(&self) -> Result<IEnumSTATDATA> {
		Err(OLE_E_ADVISENOTSUPPORTED.into())
	}
}

#[derive(Clone)]
#[implement(IEnumFORMATETC)]
struct EnumFormatEtc {
	formats: Vec<FORMATETC>,
	index: Cell<usize>,
}

impl EnumFormatEtc {
	fn new(formats: &[FORMATETC]) -> Self {
		Self {
			formats: formats.to_vec(),
			index: Cell::new(0),
		}
	}
}

impl IEnumFORMATETC_Impl for EnumFormatEtc_Impl {
	fn Next(&self, celt: u32, rgelt: *mut FORMATETC, pceltfetched: *mut u32) -> HRESULT {
		unsafe {
			if celt == 0 || rgelt.is_null() {
				return E_INVALIDARG;
			}

			let mut copied: u32 = 0;
			let mut idx = self.index.get();

			while idx < self.formats.len() && copied < celt {
				// Copy the FORMATETC; ptd is always null in our usage
				let src = &self.formats[idx];
				*rgelt.add(copied as usize) = *src;

				// Deep copy ptd if non-null (matches C++ DeepCopyFormatEtc)
				if !src.ptd.is_null() {
					let ptd = CoTaskMemAlloc(size_of::<DVTARGETDEVICE>());
					if !ptd.is_null() {
						std::ptr::copy_nonoverlapping(
							src.ptd as *const u8,
							ptd as *mut u8,
							size_of::<DVTARGETDEVICE>(),
						);
						(*rgelt.add(copied as usize)).ptd = ptd as *mut DVTARGETDEVICE;
					}
				}

				copied += 1;
				idx += 1;
			}

			self.index.set(idx);

			if !pceltfetched.is_null() {
				*pceltfetched = copied;
			}

			if copied == celt { S_OK } else { S_FALSE }
		}
	}

	fn Skip(&self, celt: u32) -> Result<()> {
		let new_index = self.index.get() + celt as usize;
		self.index.set(new_index);
		if new_index <= self.formats.len() {
			Ok(())
		} else {
			Err(S_FALSE.into())
		}
	}

	fn Reset(&self) -> Result<()> {
		self.index.set(0);
		Ok(())
	}

	fn Clone(&self) -> Result<IEnumFORMATETC> {
		let clone = EnumFormatEtc {
			formats: self.formats.clone(),
			index: Cell::new(self.index.get()),
		};
		Ok(clone.into())
	}
}

pub fn start_drag_drop(paths: &[String], allowed_effects: DROPEFFECT) -> Result<DROPEFFECT> {
	unsafe {
		OleInitialize(None)?;

		let drop_source: IDropSource = DropSource.into();
		let data_object: IDataObject = DataObject::new(paths.to_vec()).into();

		let mut effect = DROPEFFECT_NONE;
		let result = DoDragDrop(&data_object, &drop_source, allowed_effects, &mut effect);

		OleUninitialize();

		if result.is_err() {
			return Err(result.into());
		}
		Ok(effect)
	}
}
