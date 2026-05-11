use std::{ffi::CString, os::raw::c_int, ptr, ptr::NonNull, slice, sync::Arc};

use libass_sys as ffi;

use crate::{Result, track::Track};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DefaultFontProvider {
    None,
    Autodetect,
    CoreText,
    Fontconfig,
    DirectWrite,
}

pub fn version() -> i32 {
    unsafe { ffi::ass_library_version() }
}

pub(crate) struct RawLibrary {
    handle: NonNull<ffi::ass_library>,
}

impl RawLibrary {
    pub fn new() -> Result<Self> {
        let lib = unsafe { ffi::ass_library_init() };

        Ok(RawLibrary {
            handle: NonNull::new(lib).ok_or(crate::Error)?,
        })
    }

    pub fn as_ptr(&self) -> *mut ffi::ass_library {
        self.handle.as_ptr()
    }
}

unsafe impl Send for RawLibrary {}
unsafe impl Sync for RawLibrary {}

impl Drop for RawLibrary {
    fn drop(&mut self) {
        unsafe {
            ffi::ass_library_done(self.handle.as_ptr());
        }
    }
}

#[derive(Clone)]
pub struct Library {
    pub(crate) raw: Arc<RawLibrary>,
}

impl Library {
    #[doc(alias = "ass_library_init")]
    pub fn new() -> Result<Self> {
        RawLibrary::new().map(|raw| Self { raw: Arc::new(raw) })
    }

    #[doc(alias = "ass_set_fonts_dir")]
    pub fn set_fonts_dir(&mut self, fonts_dir: Option<&str>) {
        match fonts_dir {
            Some(fonts_dir) => {
                let fonts_dir = CString::new(fonts_dir).unwrap();
                unsafe {
                    ffi::ass_set_fonts_dir(self.raw.as_ptr(), fonts_dir.as_ptr());
                }
            }
            None => unsafe {
                ffi::ass_set_fonts_dir(self.raw.as_ptr(), ptr::null());
            },
        }
    }

    #[doc(alias = "ass_set_extract_fonts")]
    pub fn set_extract_fonts(&mut self, extract: bool) {
        unsafe {
            ffi::ass_set_extract_fonts(self.raw.as_ptr(), c_int::from(extract));
        }
    }

    #[doc(alias = "ass_set_style_overrides")]
    pub fn set_style_overrides<'a, I>(&mut self, list: I)
    where
        I: IntoIterator<Item = &'a str>,
    {
        let c_strings: Vec<CString> = list.into_iter().map(|s| CString::new(s).unwrap()).collect();
        let mut c_strs: Vec<*mut i8> = c_strings
            .iter()
            .map(|c_string| c_string.as_ptr().cast_mut())
            .collect();
        unsafe {
            ffi::ass_set_style_overrides(self.raw.as_ptr(), c_strs.as_mut_ptr());
        }
    }

    #[doc(alias = "ass_add_font")]
    pub fn add_font(&mut self, name: &str, data: &[u8]) {
        let name = CString::new(name).unwrap();
        unsafe {
            ffi::ass_add_font(
                self.raw.as_ptr(),
                name.as_ptr(),
                data.as_ptr().cast(),
                data.len() as c_int,
            );
        }
    }

    #[doc(alias = "ass_clear_fonts")]
    pub fn clear_fonts(&mut self) {
        unsafe {
            ffi::ass_clear_fonts(self.raw.as_ptr());
        }
    }

    #[doc(alias = "ass_get_available_font_providers")]
    pub fn available_font_providers(&self) -> Vec<DefaultFontProvider> {
        let mut ptr: *mut ffi::ASS_DefaultFontProvider = ptr::null_mut();
        let mut size: usize = 0;

        unsafe {
            ffi::ass_get_available_font_providers(self.raw.as_ptr(), &raw mut ptr, &raw mut size);
        }

        let providers = unsafe { slice::from_raw_parts(ptr, size) }
            .iter()
            .map(|provider| {
                use ffi::ASS_DefaultFontProvider::{
                    ASS_FONTPROVIDER_AUTODETECT, ASS_FONTPROVIDER_CORETEXT,
                    ASS_FONTPROVIDER_DIRECTWRITE, ASS_FONTPROVIDER_FONTCONFIG,
                    ASS_FONTPROVIDER_NONE,
                };

                use crate::library::DefaultFontProvider::{
                    Autodetect, CoreText, DirectWrite, Fontconfig, None,
                };

                match provider {
                    ASS_FONTPROVIDER_NONE => None,
                    ASS_FONTPROVIDER_AUTODETECT => Autodetect,
                    ASS_FONTPROVIDER_CORETEXT => CoreText,
                    ASS_FONTPROVIDER_FONTCONFIG => Fontconfig,
                    ASS_FONTPROVIDER_DIRECTWRITE => DirectWrite,
                }
            })
            .collect();

        unsafe {
            ffi::ass_free(ptr.cast());
        }

        providers
    }

    pub fn new_track(&mut self) -> Result<Track> {
        let track = unsafe { ffi::ass_new_track(self.raw.as_ptr()) };

        if track.is_null() {
            return Err(crate::Error);
        }

        unsafe { Ok(Track::new_unchecked(track, self.raw.clone())) }
    }

    pub fn new_track_from_file(&mut self, filename: &str, codepage: &str) -> Result<Track> {
        let filename = CString::new(filename).unwrap();
        let cp = CString::new(codepage).unwrap();
        let track =
            unsafe { ffi::ass_read_file(self.raw.as_ptr(), filename.as_ptr(), cp.as_ptr()) };

        if track.is_null() {
            return Err(crate::Error);
        }

        unsafe { Ok(Track::new_unchecked(track, self.raw.clone())) }
    }

    pub fn new_track_from_memory(&mut self, data: &mut [u8], codepage: &str) -> Result<Track> {
        let cp = CString::new(codepage).unwrap();
        let track = unsafe {
            ffi::ass_read_memory(
                self.raw.as_ptr(),
                data.as_mut_ptr().cast(),
                data.len(),
                cp.as_ptr(),
            )
        };

        if track.is_null() {
            return Err(crate::Error);
        }

        unsafe { Ok(Track::new_unchecked(track, self.raw.clone())) }
    }
}
