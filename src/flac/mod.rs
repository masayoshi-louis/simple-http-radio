use std::os::raw::{c_uint, c_void};
use std::ptr;

mod c_api;

pub struct StreamEncoder {
    inner: *mut c_api::FLAC__StreamEncoder,
    ok: bool,
}

impl StreamEncoder {
    #[inline]
    pub fn create() -> StreamEncoder {
        let handle = unsafe { c_api::FLAC__stream_encoder_new() };
        StreamEncoder {
            inner: handle,
            ok: true,
        }
    }

    #[inline]
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    #[inline]
    pub fn set_verify(&mut self, value: bool) {
        if !self.is_ok() {
            return;
        }
        unsafe {
            self.ok = c_api::FLAC__stream_encoder_set_verify(self.inner, value as i32) != 0
        }
    }

    #[inline]
    pub fn set_compression_level(&mut self, value: u32) {
        if !self.is_ok() {
            return;
        }
        unsafe {
            self.ok = c_api::FLAC__stream_encoder_set_compression_level(self.inner, value) != 0
        }
    }

    #[inline]
    pub fn set_channels(&mut self, value: u32) {
        if !self.is_ok() {
            return;
        }
        unsafe {
            self.ok = c_api::FLAC__stream_encoder_set_channels(self.inner, value) != 0
        }
    }

    #[inline]
    pub fn set_bits_per_sample(&mut self, value: u32) {
        if !self.is_ok() {
            return;
        }
        unsafe {
            self.ok = c_api::FLAC__stream_encoder_set_bits_per_sample(self.inner, value) != 0
        }
    }

    #[inline]
    pub fn set_sample_rate(&mut self, value: u32) {
        if !self.is_ok() {
            return;
        }
        unsafe {
            self.ok = c_api::FLAC__stream_encoder_set_sample_rate(self.inner, value) != 0
        }
    }

    #[inline]
    pub fn set_total_samples_estimate(&mut self, value: u64) {
        if !self.is_ok() {
            return;
        }
        unsafe {
            self.ok = c_api::FLAC__stream_encoder_set_total_samples_estimate(self.inner, value) != 0
        }
    }

    #[inline]
    pub fn process_interleaved(&mut self, buffer: &[i32], samples: usize) {
        if !self.is_ok() {
            return;
        }
        unsafe {
            self.ok = c_api::FLAC__stream_encoder_process_interleaved(self.inner, &buffer[0], samples as c_uint) != 0
        }
    }

    #[inline]
    pub fn init_ogg_stream_non_seekable<F>(&mut self, write_cb: &mut F)
        where F: FnMut(&[u8], usize, usize) -> bool {
        if !self.is_ok() {
            return;
        }
        unsafe {
            self.ok = c_api::FLAC__stream_encoder_init_ogg_stream(
                self.inner,
                None,
                Some(Self::write_cb::<F>),
                None,
                None,
                None,
                write_cb as *const _ as *mut c_void,
            ) != 0
        }
    }

    #[inline]
    pub fn finish(&mut self) {
        if !self.is_ok() {
            return;
        }
        unsafe {
            self.ok = c_api::FLAC__stream_encoder_finish(self.inner) != 0;
        }
    }

    unsafe
    extern "C"
    fn write_cb<F>(encoder: *const c_api::FLAC__StreamEncoder,
                   buffer: *const u8,
                   bytes: usize,
                   samples: c_uint,
                   current_frame: c_uint,
                   client_data: *mut c_void,
    ) -> c_api::FLAC__StreamEncoderWriteStatus
        where F: FnMut(&[u8], usize, usize) -> bool {
        let cb = &mut *(client_data as *mut F);
        cb(
            std::slice::from_raw_parts(buffer, bytes),
            samples as usize,
            current_frame as usize,
        ) as c_api::FLAC__StreamEncoderWriteStatus
    }
}

impl Drop for StreamEncoder {
    fn drop(&mut self) {
        if self.inner != ptr::null_mut() {
            unsafe {
                c_api::FLAC__stream_encoder_delete(self.inner);
            }
            self.inner = ptr::null_mut();
        }
    }
}
