use std::{
    ffi::c_void,
    fs::File,
    os::{fd::AsRawFd, raw::c_int},
    path::Path,
    ptr::{self, NonNull},
};

pub struct MemoryMappedFile {
    ptr: NonNull<c_void>,
    len: usize,
}

impl MemoryMappedFile {
    pub fn open<P>(path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let len = file.metadata()?.len() as usize;

        let map = unsafe {
            libc::mmap(
                ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            )
        };

        if map == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error().into());
        }

        let ptr = NonNull::new(map).ok_or(anyhow::anyhow!("Mmap failed to map!"))?;

        Ok(Self { ptr, len })
    }

    pub fn lines(&self) -> Lines<'_> {
        Lines::new(self.as_ref())
    }
}

impl Drop for MemoryMappedFile {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.ptr.as_ptr(), self.len) };
    }
}

impl AsRef<[u8]> for MemoryMappedFile {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.len) }
    }
}

pub struct Lines<'a> {
    remaining: &'a [u8],
}

impl<'a> Lines<'a> {
    fn new(items: &'a [u8]) -> Self {
        Self { remaining: items }
    }
}

impl<'a> Iterator for Lines<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        if let Some(index) = memchr(self.remaining, b'\n') {
            let result = &self.remaining[..index];
            self.remaining = &self.remaining[index + 1..];
            Some(result)
        } else {
            let result = self.remaining;
            self.remaining = &self.remaining[self.remaining.len()..];
            Some(result)
        }
    }
}

pub fn split_by(slice: &[u8], c: u8) -> (&[u8], &[u8]) {
    if let Some(index) = memchr(slice, c) {
        (&slice[..index], &slice[index + 1..])
    } else {
        (slice, &slice[slice.len()..])
    }
}

#[inline(always)]
fn memchr(haystack: &[u8], needle: u8) -> Option<usize> {
    let ptr = unsafe {
        libc::memchr(
            haystack.as_ptr() as *const c_void,
            needle as c_int,
            haystack.len(),
        )
    };

    if ptr.is_null() {
        return None;
    } else {
        let len = unsafe { ptr.offset_from(haystack.as_ptr() as *const c_void) } as usize;
        return Some(len);
    }
}
