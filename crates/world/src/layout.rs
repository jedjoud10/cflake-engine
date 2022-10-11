/*
pub trait RefSlice<'i> {
    type ItemRef: 'i;
    type Item: 'static;
    type Ptr: 'static + Copy;

    fn as_ptr(&self) -> Self::Ptr;
    unsafe fn from_ptr(ptr: Self::Ptr, len: usize) -> Self;

    fn fetch<'s2: 'i>(&'s2 self, index: usize) -> Option<Self::ItemRef>;
}

impl<'s, 'i, T: 'static> RefSlice<'i> for &'s mut [T] {
    type ItemRef = &'i mut T;
    type Item = T;

    fn as_ptr(&self) -> Self::Ptr {
        self.as_ptr()
    }

    fn from_ptr(ptr: Self::Ptr) -> Self {
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    fn fetch<'s2: 'i>(&'s2 self, index: usize) -> Option<Self::ItemRef>{
        self.get_mut(index)
    }
}

impl<'s, 'i, T: 'static> RefSlice<'i> for Option<&'s mut [T]> {
    type ItemRef = Option<&'i mut T>;
    type Item = T;

    fn fetch<'s2: 'i>(&'s2 self, index: usize) -> Option<Self::ItemRef> {
        self.as_mut().map(|slice| slice.get_mut(index))
    }
}

*/