use std::{
    alloc::{Layout, alloc, dealloc},
    fmt::Display,
    mem::ManuallyDrop,
    ptr::null,
};

pub unsafe extern "C" fn dealloc_safe_string(data: *mut u8, len: usize) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(len, align_of::<u8>());
        dealloc(data, layout);
    }
}
#[repr(C)]
pub struct SafeString {
    data: *const u8,
    len: usize,
    dealloc: unsafe extern "C" fn(*mut u8, usize),
}
impl From<&str> for SafeString {
    fn from(value: &str) -> Self {
        unsafe {
            let len = value.len();
            let layout = Layout::from_size_align_unchecked(len, align_of::<u8>());
            let data = alloc(layout);
            std::ptr::copy(value.as_ptr(), data, len);
            Self {
                data,
                len,
                dealloc: dealloc_safe_string,
            }
        }
    }
}
impl From<String> for SafeString {
    fn from(value: String) -> Self {
        let s = Box::leak(value.into_boxed_str());
        let layout_old = Layout::for_value(s);
        let len = layout_old.size();
        debug_assert!(
            layout_old.align() == align_of::<u8>(),
            "String alignment is wrong. Could not safely create layout"
        );
        let data = s.as_ptr();
        Self {
            data,
            len,
            dealloc: dealloc_safe_string,
        }
    }
}
impl Display for SafeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            let s = std::slice::from_raw_parts(self.data, self.len);
            let s = str::from_utf8_unchecked(s);
            write!(f, "{s}")
        }
    }
}
impl Drop for SafeString {
    fn drop(&mut self) {
        unsafe { (self.dealloc)(self.data as *mut u8, self.len) };
        self.data = null();
        self.len = 0usize;
    }
}

#[repr(C)]
union SafeResultData<T, E> {
    ok: ManuallyDrop<T>,
    err: ManuallyDrop<E>,
}

/// Requires that T and E are both FFI stable (#[repr(C)] is a good way to all but guarantee this)
///
/// Otherwise, risks segfaults
#[repr(C)]
pub struct SafeResult<T, E> {
    code: i32,
    data: SafeResultData<T, E>,
}
impl<T, E> SafeResult<T, E> {
    pub fn ok(val: T) -> Self {
        Self {
            code: 0,
            data: SafeResultData {
                ok: ManuallyDrop::new(val),
            },
        }
    }
    pub fn err(err: E, code: i32) -> Self {
        Self {
            code,
            data: SafeResultData {
                err: ManuallyDrop::new(err),
            },
        }
    }

    pub fn to_result(mut self) -> Result<T, E> {
        unsafe {
            if self.code == 0 {
                let ok = ManuallyDrop::take(&mut self.data.ok);
                Ok(ok)
            } else {
                let err = ManuallyDrop::take(&mut self.data.err);
                Err(err)
            }
        }
    }
}
impl<T, E> Drop for SafeResult<T, E> {
    fn drop(&mut self) {
        unsafe {
            if self.code == 0 {
                ManuallyDrop::drop(&mut self.data.ok);
            } else {
                ManuallyDrop::drop(&mut self.data.err);
            }
        }
    }
}
