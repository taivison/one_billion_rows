use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
    ops::Deref,
};

#[repr(transparent)]
pub struct Array(ReprArray);

impl Array {
    #[inline]
    pub fn new<T>(items: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self(ReprArray::new(items))
    }
}

impl From<&[u8]> for Array {
    fn from(value: &[u8]) -> Self {
        Array::new(value)
    }
}

impl AsRef<[u8]> for Array {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match &self.0 {
            ReprArray::Inline { len, buf } => &buf[..*len as usize],
            ReprArray::Heap(items) => items,
        }
    }
}

impl Deref for Array {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Borrow<[u8]> for Array {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

impl Hash for Array {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl Eq for Array {}
impl PartialEq for Array {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Ord for Array {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}
impl PartialOrd for Array {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

enum ReprArray {
    Inline {
        len: InlineLen,
        buf: [u8; INLINE_CAP],
    },
    Heap(Box<[u8]>),
}

const INLINE_CAP: usize = InlineLen::_V23 as usize;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InlineLen {
    _V0 = 0,
    _V1,
    _V2,
    _V3,
    _V4,
    _V5,
    _V6,
    _V7,
    _V8,
    _V9,
    _V10,
    _V11,
    _V12,
    _V13,
    _V14,
    _V15,
    _V16,
    _V17,
    _V18,
    _V19,
    _V20,
    _V21,
    _V22,
    _V23,
}

impl ReprArray {
    pub fn new<T>(items: T) -> Self
    where
        T: AsRef<[u8]>,
    {
        let items = items.as_ref();
        if items.len() <= INLINE_CAP {
            let mut buf = [0; INLINE_CAP];
            buf[..items.len()].copy_from_slice(items);
            return Self::Inline {
                len: unsafe { std::mem::transmute(items.len() as u8) },
                buf,
            };
        } else {
            let mut heap = Box::new_uninit_slice(items.len());
            _ = heap.write_copy_of_slice(items);

            return Self::Heap(unsafe { heap.assume_init() });
        }
    }
}
