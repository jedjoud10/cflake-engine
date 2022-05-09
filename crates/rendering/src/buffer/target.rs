use std::{num::NonZeroU32, marker::PhantomData};

use num::{Integer, Bounded};

use super::GPUSendable;

// Trait implemented for buffer targets
pub trait BufferTarget {
    // The buer target type
    const GL_TARGET: NonZeroU32;

    // The element type that will be stored within the buffer
    type ElemType: GPUSendable;
}

// Le fonction cause I don't wanna make it look ugly
const fn ct(gltype: u32) -> NonZeroU32 {
    unsafe { NonZeroU32::new_unchecked(gltype) }
}

// Empty structs for buffer targets indeed
pub struct ArrayBuffer<T: GPUSendable>(PhantomData<*const T>);
pub struct ElementBuffer<I: Integer + Bounded + GPUSendable>(PhantomData<*const I>);
pub struct AtomicBuffer(());

// Implementations of the buffer target
impl<T: GPUSendable> BufferTarget for ArrayBuffer<T> {
    const GL_TARGET: NonZeroU32 = ct(gl::ARRAY_BUFFER);
    type ElemType = T;
}

impl<I: Integer + Bounded + GPUSendable> BufferTarget for ElementBuffer<I> {
    const GL_TARGET: NonZeroU32 = ct(gl::ELEMENT_ARRAY_BUFFER);
    type ElemType = I;
}

impl BufferTarget for AtomicBuffer {
    const GL_TARGET: NonZeroU32 = ct(gl::ATOMIC_COUNTER_BUFFER);
    type ElemType = u32;
}