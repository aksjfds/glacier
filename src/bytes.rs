use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    ptr::NonNull,
    slice::{from_raw_parts, from_raw_parts_mut},
    usize,
};

pub struct Bytes {
    ptr: NonNull<u8>,
    cap: usize,
    len: usize,
}

unsafe impl Sync for Bytes {}
unsafe impl Send for Bytes {}

impl Bytes {
    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::array::<u8>(cap).unwrap();
        let ptr = unsafe { alloc(layout) };
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Bytes { ptr, cap, len: 0 }
    }

    pub fn grow(&mut self) {
        let old_ptr = self.ptr.as_ptr();
        let old_layout = Layout::array::<u8>(self.cap).unwrap();
        let new_ptr = unsafe { realloc(old_ptr, old_layout, self.cap * 2) };

        self.ptr = NonNull::new(new_ptr).unwrap();
        self.cap *= 2;
    }

    pub fn as_slice(&self) -> &[u8] {
        let ptr = self.ptr.as_ptr();
        unsafe { from_raw_parts(ptr, self.len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        let ptr = self.ptr.as_ptr();
        unsafe { from_raw_parts_mut(ptr, self.len) }
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_slice()) }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Drop for Bytes {
    fn drop(&mut self) {
        if self.cap != 0 {
            let layout = Layout::array::<u8>(self.cap).unwrap();
            unsafe { dealloc(self.ptr.as_ptr(), layout) };
        }
    }
}

impl Bytes {
    pub fn get_32_space(&mut self) -> &mut [u8] {
        if self.cap - self.len < 32 {
            self.grow();
            self.get_32_space()
        } else {
            unsafe {
                let ptr = self.ptr.as_ptr().add(self.len);
                from_raw_parts_mut(ptr, 32)
            }
        }
    }

    pub fn get_free_space(&mut self) -> &mut [u8] {
        if self.cap - self.len < 32 {
            self.grow();
            self.get_free_space()
        } else {
            unsafe {
                let ptr = self.ptr.as_ptr().add(self.len);
                // from_raw_parts_mut(ptr, 32)
                from_raw_parts_mut(ptr, self.cap - self.len)
            }
        }
    }

    pub fn modify_len(&mut self, len: usize) {
        self.len += len;
    }
}

impl Bytes {
    pub fn is_end(&self) -> bool {
        if self.len < 4 {
            return false;
        }

        let end = &self.as_slice()[self.len - 4..];
        if is_lf(&end[..2]) && is_lf(&end[2..]) {
            true
        } else {
            false
        }
    }

    pub fn first_line(&self) -> Option<&[u8]> {
        let slice = self.as_slice();
        for pos in 0..self.len {
            let temp = &[slice[pos], slice[pos + 1]];
            if is_lf(temp) {
                return Some(&slice[0..pos]);
            }
        }

        None
    }
}

pub fn is_lf(slice: &[u8]) -> bool {
    match slice {
        b"\r\n" => true,
        _ => false,
    }
}

#[test]
fn test() {
    let a = [0; 10];
    let b = unsafe { from_raw_parts(a.as_ptr(), 3) };

    println!("{:#?}", b);
}
