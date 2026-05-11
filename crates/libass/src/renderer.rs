use std::{
    ffi::CString,
    os::raw::c_int,
    ptr::{self, NonNull},
    sync::Arc,
};

use libass_sys as ffi;

use crate::{
    Library, RawLibrary, Result,
    image::Image,
    library::DefaultFontProvider,
    style::{OverrideBits, Style},
    track::Track,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShapingLevel {
    Simple,
    Complex,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Hinting {
    None,
    Light,
    Normal,
    Native,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Change {
    None,
    Position,
    Content,
}

pub struct Renderer {
    handle: NonNull<ffi::ass_renderer>,
    _library: Arc<RawLibrary>,
    override_style: Option<Style>,
}

impl Renderer {
    pub fn new(library: &mut Library) -> Result<Renderer> {
        let renderer = unsafe { ffi::ass_renderer_init(library.raw.as_ptr()) };

        Ok(Renderer {
            handle: NonNull::new(renderer).ok_or(crate::Error)?,
            _library: library.raw.clone(),
            override_style: None,
        })
    }

    pub fn render_frame(&mut self, track: &mut Track, now: i64) -> (Option<Image<'_>>, Change) {
        let mut change = 0;

        let image = unsafe {
            ffi::ass_render_frame(
                self.handle.as_ptr(),
                track.as_ptr().cast_mut(),
                now,
                &raw mut change,
            )
        };

        let change = match change {
            0 => Change::None,
            1 => Change::Position,
            2 => Change::Content,
            _ => unreachable!(),
        };

        if image.is_null() {
            (None, change)
        } else {
            unsafe { (Some(Image::new_unchecked(image)), change) }
        }
    }

    pub fn set_fonts<'a>(
        &mut self,
        default_font: impl Into<Option<&'a str>>,
        default_family: impl Into<Option<&'a str>>,
        default_font_provider: DefaultFontProvider,
        fontconfig_config_path: impl Into<Option<&'a str>>,
        update_fontconfig_cache: bool,
    ) {
        use ffi::ASS_DefaultFontProvider::{
            ASS_FONTPROVIDER_AUTODETECT, ASS_FONTPROVIDER_CORETEXT, ASS_FONTPROVIDER_DIRECTWRITE,
            ASS_FONTPROVIDER_FONTCONFIG, ASS_FONTPROVIDER_NONE,
        };

        let default_font: Option<CString> = default_font.into().map(|x| CString::new(x).unwrap());
        let default_family: Option<CString> =
            default_family.into().map(|x| CString::new(x).unwrap());
        let fontconfig_config_path: Option<CString> = fontconfig_config_path
            .into()
            .map(|x| CString::new(x).unwrap());

        let default_font_provider = match default_font_provider {
            DefaultFontProvider::None => ASS_FONTPROVIDER_NONE,
            DefaultFontProvider::Autodetect => ASS_FONTPROVIDER_AUTODETECT,
            DefaultFontProvider::CoreText => ASS_FONTPROVIDER_CORETEXT,
            DefaultFontProvider::Fontconfig => ASS_FONTPROVIDER_FONTCONFIG,
            DefaultFontProvider::DirectWrite => ASS_FONTPROVIDER_DIRECTWRITE,
        };

        let default_font = default_font.as_ref().map_or(ptr::null(), |s| s.as_ptr());
        let default_family = default_family.as_ref().map_or(ptr::null(), |s| s.as_ptr());
        let config = fontconfig_config_path
            .as_ref()
            .map_or(ptr::null(), |s| s.as_ptr());

        unsafe {
            ffi::ass_set_fonts(
                self.handle.as_ptr(),
                default_font,
                default_family,
                default_font_provider as c_int,
                config,
                c_int::from(update_fontconfig_cache),
            );
        }
    }

    pub fn set_frame_size(&mut self, width: i32, height: i32) {
        unsafe { ffi::ass_set_frame_size(self.handle.as_ptr(), width, height) }
    }

    pub fn set_storage_size(&mut self, width: i32, height: i32) {
        unsafe { ffi::ass_set_storage_size(self.handle.as_ptr(), width, height) }
    }

    pub fn set_shaper(&mut self, level: ShapingLevel) {
        unsafe {
            use ffi::ASS_ShapingLevel::{ASS_SHAPING_COMPLEX, ASS_SHAPING_SIMPLE};

            use crate::renderer::ShapingLevel::{Complex, Simple};

            ffi::ass_set_shaper(self.handle.as_ptr(), {
                match level {
                    Simple => ASS_SHAPING_SIMPLE,
                    Complex => ASS_SHAPING_COMPLEX,
                }
            });
        }
    }

    pub fn set_margins(&mut self, top: i32, bottom: i32, left: i32, right: i32) {
        unsafe { ffi::ass_set_margins(self.handle.as_ptr(), top, bottom, left, right) }
    }

    pub fn use_margins(&mut self, use_: bool) {
        unsafe { ffi::ass_set_use_margins(self.handle.as_ptr(), c_int::from(use_)) }
    }

    pub fn set_pixel_aspect_ratio(&mut self, par: f64) {
        unsafe { ffi::ass_set_pixel_aspect(self.handle.as_ptr(), par) }
    }

    pub fn set_aspect_ratio(&mut self, dar: f64, sar: f64) {
        unsafe { ffi::ass_set_aspect_ratio(self.handle.as_ptr(), dar, sar) }
    }

    pub fn set_font_scale(&mut self, font_scale: f64) {
        unsafe { ffi::ass_set_font_scale(self.handle.as_ptr(), font_scale) }
    }

    pub fn set_hinting(&mut self, font_hinting: Hinting) {
        unsafe {
            use ffi::ASS_Hinting::{
                ASS_HINTING_LIGHT, ASS_HINTING_NATIVE, ASS_HINTING_NONE, ASS_HINTING_NORMAL,
            };

            use crate::Hinting::{Light, Native, None, Normal};

            let ht = match font_hinting {
                None => ASS_HINTING_NONE,
                Light => ASS_HINTING_LIGHT,
                Normal => ASS_HINTING_NORMAL,
                Native => ASS_HINTING_NATIVE,
            };

            ffi::ass_set_hinting(self.handle.as_ptr(), ht);
        }
    }

    pub fn set_line_spacing(&mut self, line_spacing: f64) {
        unsafe { ffi::ass_set_line_spacing(self.handle.as_ptr(), line_spacing) }
    }

    pub fn set_line_position(&mut self, line_position: f64) {
        unsafe { ffi::ass_set_line_position(self.handle.as_ptr(), line_position) }
    }

    pub fn set_cache_limits(&mut self, glyph_max: i32, bitmap_max_size: i32) {
        unsafe { ffi::ass_set_cache_limits(self.handle.as_ptr(), glyph_max, bitmap_max_size) }
    }

    pub fn set_selective_style_override(&mut self, style: &Style) {
        self.override_style = Some(style.clone());
        let ass_style = unsafe { self.override_style.as_ref().unwrap().as_ass_style() };

        unsafe {
            ffi::ass_set_selective_style_override(
                self.handle.as_ptr(),
                ptr::from_ref(&ass_style).cast_mut(),
            );
        }
    }

    pub fn set_selective_style_override_enabled(&mut self, bits: OverrideBits) {
        unsafe {
            ffi::ass_set_selective_style_override_enabled(
                self.handle.as_ptr(),
                bits.bits().cast_signed(),
            );
        }
    }

    #[doc(hidden)]
    pub fn update_fonts(&mut self) -> std::result::Result<(), i32> {
        let ret = unsafe { ffi::ass_fonts_update(self.handle.as_ptr()) };
        if ret == 0 { Ok(()) } else { Err(ret) }
    }
}

unsafe impl Send for Renderer {}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { ffi::ass_renderer_done(self.handle.as_ptr()) }
    }
}
