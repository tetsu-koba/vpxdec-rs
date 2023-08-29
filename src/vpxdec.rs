use std::error::Error;

pub struct VpxDec {
    codec: vpx_sys::vpx_codec_ctx_t,
    iter: vpx_sys::vpx_codec_iter_t,
}

impl VpxDec {
    pub fn init(fourcc: &[u8; 4]) -> Result<Self, Box<dyn Error>> {
        let mut codec = vpx_sys::vpx_codec_ctx_t {
            config: vpx_sys::vpx_codec_ctx__bindgen_ty_1 { raw: 0 as _ },
            err: vpx_sys::vpx_codec_err_t::VPX_CODEC_OK,
            err_detail: 0 as _,
            iface: 0 as _,
            init_flags: 0 as _,
            name: 0 as _,
            priv_: 0 as _,
        };
        unsafe {
            let interface = if fourcc == b"VP80" {
                vpx_sys::vpx_codec_vp8_dx()
            } else if fourcc == b"VP90" {
                vpx_sys::vpx_codec_vp9_dx()
            } else {
                return Err("Unsupported fourcc".into());
            };
            if interface == 0 as _ {
                return Err("Failed to get Vp8 interface".into());
            }
            let res = vpx_sys::vpx_codec_dec_init_ver(
                &mut codec,
                interface,
                std::ptr::null(),
                0,
                vpx_sys::VPX_DECODER_ABI_VERSION as _,
            );
            if res != vpx_sys::vpx_codec_err_t::VPX_CODEC_OK {
                return Err("Failed to initialize Vpx decoder".into());
            }
        }
        Ok(VpxDec {
            codec,
            iter: std::ptr::null(),
        })
    }

    pub fn decode(&mut self, frame_buffer: &[u8]) -> Result<(), Box<dyn Error>> {
        unsafe {
            let res = vpx_sys::vpx_codec_decode(
                &mut self.codec,
                frame_buffer.as_ptr(),
                frame_buffer.len() as _,
                0 as _,
                0 as _,
            );
            if res != vpx_sys::vpx_codec_err_t::VPX_CODEC_OK {
                return Err("Failed to initialize Vpx decoder".into());
            }
        }
        self.iter = std::ptr::null();
        Ok(())
    }

    pub fn get_frame(&mut self) -> *mut vpx_sys::vpx_image_t {
        unsafe { vpx_sys::vpx_codec_get_frame(&mut self.codec, &mut self.iter) }
    }
}

impl Drop for VpxDec {
    fn drop(&mut self) {
        unsafe {
            vpx_sys::vpx_codec_destroy(&mut self.codec);
        }
    }
}
