use std::{
    alloc::{alloc, dealloc, realloc, Layout},
    ptr::{copy, NonNull},
    slice::{from_raw_parts, from_raw_parts_mut},
    usize,
};

pub struct Bytes {
    ptr: NonNull<u8>,
    cap: usize,
    len: usize,
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
    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::array::<u8>(cap).unwrap();
        let ptr = unsafe { alloc(layout) };
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Bytes { ptr, cap, len: 0 }
    }

    pub fn push(&mut self, val: u8) {
        unsafe { self.ptr.add(self.len).write(val) };
        self.len += 1;
    }

    pub fn grow(&mut self) {
        let old_ptr = self.ptr.as_ptr();
        let old_layout = Layout::array::<u8>(self.cap).unwrap();
        let new_ptr = unsafe { realloc(old_ptr, old_layout, self.cap * 2) };

        self.ptr = NonNull::new(new_ptr).unwrap();
        self.cap *= 2;
    }

    pub fn push_slice(&mut self, slice: &[u8]) {
        let slice_len = slice.len();

        if slice_len + self.len < self.cap {
            let self_ptr = unsafe { self.ptr.as_ptr().add(self.len) };
            let slice_ptr = slice.as_ptr();
            unsafe { copy(slice_ptr, self_ptr, slice_len) };

            self.len += slice_len;
        } else {
            self.grow();
            self.push_slice(slice);
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        let ptr = self.ptr.as_ptr();
        unsafe { from_raw_parts(ptr, self.len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        let ptr = self.ptr.as_ptr();
        unsafe { from_raw_parts_mut(ptr, self.len) }
    }

    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(self.as_slice()).to_string()
    }

    pub fn is_end(&self) -> bool {
        if self.len < 4 {
            return false;
        }

        let end = &self.as_slice()[self.len - 4..];
        if is_line(&end[..2]) && is_line(&end[2..]) {
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}
pub fn is_line(slice: &[u8]) -> bool {
    if slice.len() != 2 {
        false
    } else {
        match slice {
            [b'\r', b'\n'] => true,
            _ => false,
        }
    }
}
#[test]
fn test() {
    let mut buff = Bytes::with_capacity(10);
    buff.push(10);

    buff.push_slice("\r\n\r\n".as_bytes());

    println!("{:#?}", buff.is_end());
}
