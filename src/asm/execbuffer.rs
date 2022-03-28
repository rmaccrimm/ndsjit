use std::os::raw::c_void;
use std::{io::Error, mem, ptr};

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

    pub fn call(&self, vregs: *mut u8, mem: *mut u8) {
        // Future note - windows VirtualAlloc docs mention calling FlushInstructionCache before
        // calling modified instructions in memory. Not sure if that applies here or note
        // Assuming neither of these are null, calling the generated code should be "safe"
        assert!(self.ptr != ptr::null_mut());
        assert!(vregs != ptr::null_mut());
        assert!(mem != ptr::null_mut());
        // Needs to be a C function so we can use the C calling convention(s)
        unsafe {
            let func: unsafe extern "C" fn(*mut u8, *mut u8) = mem::transmute(self.ptr);
            func(vregs, mem);
        }
    }
}

impl Drop for ExecBuffer {
    fn drop(&mut self) {
        let err = bool::from(unsafe { VirtualFree(self.ptr, self.len, MEM_RELEASE) });
        assert!(!err, "Failed to free buffer: {}", Error::last_os_error());
    }
}
