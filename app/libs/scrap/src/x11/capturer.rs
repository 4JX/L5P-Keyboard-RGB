use std::{io, ptr, slice};

use libc;

use super::ffi::*;
use super::Display;

pub struct Capturer {
    display: Display,
    shmid: i32,
    xcbid: u32,
    buffer: *const u8,

    size: usize,
    use_yuv: bool,
    yuv: Vec<u8>,
    saved_raw_data: Vec<u128>, // for faster compare and copy
}

impl Capturer {
    pub fn new(display: Display, use_yuv: bool) -> io::Result<Capturer> {
        // Calculate dimensions.

        let pixel_width = 4;
        let rect = display.rect();
        let size = (rect.w as usize) * (rect.h as usize) * pixel_width;

        // Create a shared memory segment.

        let shmid = unsafe {
            libc::shmget(
                libc::IPC_PRIVATE,
                size,
                // Everyone can do anything.
                libc::IPC_CREAT | 0o777,
            )
        };

        if shmid == -1 {
            return Err(io::Error::last_os_error());
        }

        // Attach the segment to a readable address.

        let buffer = unsafe { libc::shmat(shmid, ptr::null(), libc::SHM_RDONLY) } as *mut u8;

        if buffer as isize == -1 {
            return Err(io::Error::last_os_error());
        }

        // Attach the segment to XCB.

        let server = display.server().raw();
        let xcbid = unsafe { xcb_generate_id(server) };
        unsafe {
            xcb_shm_attach(
                server,
                xcbid,
                shmid as u32,
                0, // False, i.e. not read-only.
            );
        }

        let c = Capturer {
            display,
            shmid,
            xcbid,
            buffer,
            size,
            use_yuv,
            yuv: Vec::new(),
            saved_raw_data: Vec::new(),
        };
        Ok(c)
    }

    pub fn set_use_yuv(&mut self, use_yuv: bool) {
        self.use_yuv = use_yuv;
    }

    pub fn display(&self) -> &Display {
        &self.display
    }

    fn get_image(&self) {
        let rect = self.display.rect();
        unsafe {
            let request = xcb_shm_get_image_unchecked(
                self.display.server().raw(),
                self.display.root(),
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                !0,
                XCB_IMAGE_FORMAT_Z_PIXMAP,
                self.xcbid,
                0,
            );
            let response =
                xcb_shm_get_image_reply(self.display.server().raw(), request, ptr::null_mut());
            libc::free(response as *mut _);
        }
    }

    pub fn frame<'b>(&'b mut self) -> std::io::Result<&'b [u8]> {
        self.get_image();
        let result = unsafe { slice::from_raw_parts(self.buffer, self.size) };
        crate::would_block_if_equal(&mut self.saved_raw_data, result)?;
        Ok(if self.use_yuv {
            crate::common::bgra_to_i420(self.display.w(), self.display.h(), &result, &mut self.yuv);
            &self.yuv[..]
        } else {
            result
        })
    }
}

impl Drop for Capturer {
    fn drop(&mut self) {
        unsafe {
            // Detach segment from XCB.
            xcb_shm_detach(self.display.server().raw(), self.xcbid);
            // Detach segment from our space.
            libc::shmdt(self.buffer as *mut _);
            // Destroy the shared memory segment.
            libc::shmctl(self.shmid, libc::IPC_RMID, ptr::null_mut());
        }
    }
}
