use std::io::Error;
use std::os::raw::c_void;
use std::ptr;

use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, VirtualProtect, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
    PAGE_EXECUTE_READ, PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

// An executable buffer
pub struct ExecBuffer {
    pub ptr: *mut c_void,
    pub len: usize,
}

impl ExecBuffer {
    pub fn from_vec(code: Vec<u8>) -> Result<ExecBuffer, Error> {
        unsafe {
            let mut system_info = SYSTEM_INFO::default();
            GetSystemInfo(&mut system_info as *mut SYSTEM_INFO);

            let buf = VirtualAlloc(
                ptr::null(),
                system_info.dwPageSize as usize,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_READWRITE,
            );
            if buf == ptr::null_mut() {
                return Err(Error::last_os_error());
            }

            ptr::copy(code.as_ptr(), buf as *mut u8, code.len());
            let mut dummy = PAGE_PROTECTION_FLAGS::default();
            VirtualProtect(
                buf,
                code.len(),
                PAGE_EXECUTE_READ,
                &mut dummy as *mut PAGE_PROTECTION_FLAGS,
            );

            if buf == ptr::null_mut() {
                Err(Error::last_os_error())
            } else {
                Ok(ExecBuffer {
                    ptr: buf,
                    len: code.len(),
                })
            }
        }
    }
}

impl Drop for ExecBuffer {
    fn drop(&mut self) {
        let err = bool::from(unsafe { VirtualFree(self.ptr, self.len, MEM_RELEASE) });
        assert!(!err, "Failed to free buffer: {}", Error::last_os_error());
    }
}
