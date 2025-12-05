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

        let next_line = unsafe {
            libc::memchr(
                self.remaining.as_ptr() as *const c_void,
                b'\n' as c_int,
                self.remaining.len(),
            )
        };

        if next_line.is_null() {
            let result = self.remaining;
            self.remaining = &self.remaining[self.remaining.len()..];
            Some(result)
        } else {
            let len =
                unsafe { next_line.offset_from(self.remaining.as_ptr() as *const c_void) } as usize;
            let result = &self.remaining[..len];
            self.remaining = &self.remaining[len + 1..];
            Some(result)
        }
    }
}

pub fn split_by(slice: &[u8], c: u8) -> (&[u8], &[u8]) {
    let ptr = unsafe { libc::memchr(slice.as_ptr() as *const c_void, c as c_int, slice.len()) };

    if ptr.is_null() {
        (slice, &slice[slice.len()..])
    } else {
        let len = unsafe { ptr.offset_from(slice.as_ptr() as *const c_void) } as usize;

        (&slice[..len], &slice[len + 1..])
    }
}
