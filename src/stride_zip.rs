use crate::iters::{SIMDIterator};
use crate::vecs::{Packed, Packable};
use crate::intrin::destride::*;
use crate::zip::{SIMDZippedIterable, SIMDZippedIterator, SIMDZippedObject};

pub struct StrideZip<T> where T : SIMDIterator, T::Vector : Destride {
    base: usize,
    peek: Option<T::Vector>,
    iter: T
}

/// A trait which can transform a collection of iterators into a `Zip`
pub trait IntoStrideZip : Sized {
    /// Return an iterator which may iterate over `self` in lockstep.
    fn stride_zip(self) -> StrideZip<Self>
        where Self : SIMDIterator, Self::Vector : Destride;
}

impl<T> IntoStrideZip for T where T : SIMDIterator, T::Vector : Destride {
    fn stride_zip(self) -> StrideZip<Self> {
        StrideZip {
            base: self.scalar_pos(),
            peek: None,
            iter: self
        }
    }
}

impl<T> SIMDZippedObject for StrideZip<T> where T : SIMDIterator, T::Vector : Destride {
    type Scalars = (T::Scalar, T::Scalar);
    type Vectors = (T::Vector, T::Vector);

    /// Return the vector length of this object.
    #[inline(always)]
    fn width(&self) -> usize {
        T::Vector::WIDTH
    }

    /// Return the scalar length of this object.
    #[inline(always)]
    fn size(&self) -> usize {
        T::Scalar::SIZE
    }
}

impl<T> ExactSizeIterator for StrideZip<T> where T : SIMDIterator, T::Vector : Destride {
    #[inline(always)]
    fn len(&self) -> usize {
        self.iter.len() / 2
    }
}

impl<T> SIMDZippedIterable for StrideZip<T> where T : SIMDIterator, T::Vector : Destride {
    #[inline(always)]
    fn scalar_pos(&self) -> usize {
        (self.iter.scalar_pos() - self.base) / 2
    }

    #[inline(always)]
    fn vector_pos(&self) -> usize {
        (self.iter.vector_pos() - (self.base / self.width())) / 2
    }

    #[inline(always)]
    fn scalar_len(&self) -> usize {
        self.iter.scalar_len() / 2
    }

    #[inline(always)]
    fn advance(&mut self, amount: usize) {
        self.iter.advance(2 * amount);
    }

    #[inline(always)]
    fn default(&self) -> Self::Vectors {
        (T::Vector::default(), T::Vector::default())
    }
}

impl<T> Iterator for StrideZip<T> where T : SIMDIterator, T::Vector : Destride {
    type Item = <Self as SIMDZippedObject>::Vectors;
    
    fn next(&mut self) -> Option<Self::Item> {
        let first = self.iter.next()?;
        let second = self.iter.next();
        if let Some(second) = second {
            Some(first.destride_two(second))
        } else {
            self.peek = Some(first);
            None
        }
    }
}

impl<T> SIMDZippedIterator for StrideZip<T> where T : SIMDIterator, T::Vector : Destride {
    fn end(&mut self) -> Option<(Self::Vectors, usize)> {
        let first = self.iter.next();
        let (end, n) = self.iter.end().unwrap_or((self.iter.default(), 0));
        if let Some(first) = first {
            Some((first.destride_two(end), (self.width() + n) / 2))
        } else {
            if let Some(v) = self.peek {
                self.peek = None;
                Some((v.destride_two(end), (self.width() + n) / 2))
            } else if n > 0 {
                Some((end.destride_two(self.iter.default()), n / 2))
            } else {
                None
            }
        }
    }
}
