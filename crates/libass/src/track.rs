use std::{ffi::CStr, os::raw::c_int, ptr::NonNull, sync::Arc};

use libass_sys as ffi;

use crate::RawLibrary;

pub struct Track {
    handle: NonNull<ffi::ass_track>,
    _library: Arc<RawLibrary>,
}

impl Track {
    pub(crate) unsafe fn new_unchecked(
        track: *mut ffi::ass_track,
        library: Arc<RawLibrary>,
    ) -> Self {
        Track {
            handle: unsafe { NonNull::new_unchecked(track) },
            _library: library,
        }
    }

    pub(crate) fn as_ptr(&self) -> *const ffi::ass_track {
        self.handle.as_ptr()
    }

    pub fn new_style(&mut self) -> StyleHandle<'_> {
        StyleHandle {
            id: unsafe { ffi::ass_alloc_style(self.handle.as_ptr()) },
            track: self,
        }
    }

    pub fn new_event(&mut self) -> Event<'_> {
        Event {
            id: unsafe { ffi::ass_alloc_event(self.handle.as_ptr()) },
            track: self,
        }
    }

    pub fn step_sub(&mut self, now: i64, movement: i32) -> i64 {
        unsafe { ffi::ass_step_sub(self.handle.as_ptr().cast(), now, movement) }
    }

    pub fn process_force_style(&mut self) {
        unsafe { ffi::ass_process_force_style(self.handle.as_ptr()) }
    }

    pub fn read_styles(&mut self, filename: &CStr, codepage: &CStr) {
        unsafe {
            ffi::ass_read_styles(self.handle.as_ptr(), filename.as_ptr(), codepage.as_ptr());
        }
    }

    pub fn set_check_readorder(&mut self, check_readorder: bool) {
        unsafe { ffi::ass_set_check_readorder(self.handle.as_ptr(), c_int::from(check_readorder)) }
    }

    pub fn flush_events(&mut self) {
        unsafe { ffi::ass_flush_events(self.handle.as_ptr()) }
    }

    pub fn process_data(&mut self, data: &[u8]) {
        unsafe {
            ffi::ass_process_data(
                self.handle.as_ptr(),
                data.as_ptr().cast(),
                data.len() as c_int,
            );
        }
    }

    pub fn process_codec_private(&mut self, data: &[u8]) {
        unsafe {
            ffi::ass_process_codec_private(
                self.handle.as_ptr(),
                data.as_ptr().cast(),
                data.len() as c_int,
            );
        }
    }

    pub fn process_chunk(&mut self, data: &[u8], timecode: i64, duration: i64) {
        unsafe {
            ffi::ass_process_chunk(
                self.handle.as_ptr(),
                data.as_ptr().cast(),
                data.len() as c_int,
                timecode,
                duration,
            );
        }
    }
}

unsafe impl Send for Track {}

impl Drop for Track {
    fn drop(&mut self) {
        unsafe { ffi::ass_free_track(self.handle.as_ptr()) }
    }
}

pub struct StyleHandle<'a> {
    pub id: i32,
    track: &'a Track,
}

impl Drop for StyleHandle<'_> {
    fn drop(&mut self) {
        unsafe { ffi::ass_free_style(self.track.handle.as_ptr(), self.id) }
    }
}

pub struct Event<'a> {
    pub id: i32,
    track: &'a Track,
}

impl Drop for Event<'_> {
    fn drop(&mut self) {
        unsafe { ffi::ass_free_event(self.track.handle.as_ptr(), self.id) }
    }
}
