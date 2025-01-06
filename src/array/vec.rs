//! [`Vec<T>`]-backed arrays
use crate::array::*;
use core::ops::{Add, Deref, DerefMut, Index, RangeBounds, Sub};

/// Arrays backed by a [`Vec<T>`].
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct VecKind {}

impl ArrayKind for VecKind {
    type Type<T> = VecArray<T>;
    type I = usize;
    type Index = VecArray<usize>;

    // A Slice for Vec is just a rust slice
    type Slice<'a, T: 'a> = &'a [T];
}

/// A newtype wrapper for [`Vec<T>`] allowing pointwise arithmetic operations.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct VecArray<T>(pub Vec<T>);

impl AsRef<<VecKind as ArrayKind>::Index> for VecArray<usize> {
    fn as_ref(&self) -> &<VecKind as ArrayKind>::Index {
        self
    }
}

// VecArray is a newtype wrapper, so we can just treat it like a regular old Vec.
impl<T> Deref for VecArray<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for VecArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Clone + PartialEq> Array<VecKind, T> for VecArray<T> {
    fn empty() -> Self {
        VecArray(Vec::default())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn concatenate(&self, other: &Self) -> Self {
        let mut result: Vec<T> = Vec::with_capacity(self.len() + other.len());
        result.extend_from_slice(self);
        result.extend_from_slice(other);
        VecArray(result)
    }

    fn fill(x: T, n: usize) -> Self {
        VecArray(vec![x; n])
    }

    fn get(&self, i: usize) -> T {
        self[i].clone()
    }

    /// Get a contiguous subrange of the array
    ///
    /// ```rust
    /// use open_hypergraphs::array::{*, vec::*};
    /// let v = VecArray(vec![0, 1, 2, 3, 4]);
    /// assert_eq!(v.get_range(..), &[0, 1, 2, 3, 4]);
    /// assert_eq!(v.get_range(..v.len()), &[0, 1, 2, 3, 4]);
    fn get_range<R: RangeBounds<usize>>(&self, rb: R) -> &[T] {
        self.index(self.to_range(rb))
    }

    fn set_range<R: RangeBounds<usize>>(&mut self, rb: R, v: &<VecKind as ArrayKind>::Type<T>) {
        let r = self.to_range(rb);
        self[r].clone_from_slice(v)
    }

    fn gather(&self, idx: &[usize]) -> Self {
        VecArray(idx.iter().map(|i| self.0[*i].clone()).collect())
    }

    fn scatter(&mut self, idx: &[usize], v: &Self) {
        for i in idx {
            self[*i] = v[*i].clone();
        }
    }
}

impl Add<&VecArray<usize>> for usize {
    type Output = VecArray<usize>;

    fn add(self, rhs: &VecArray<usize>) -> Self::Output {
        VecArray(rhs.iter().map(|x| x + self).collect())
    }
}

impl<T: Clone + Add<Output = T>> Add<VecArray<T>> for VecArray<T> {
    type Output = VecArray<T>;

    fn add(self, rhs: VecArray<T>) -> VecArray<T> {
        assert_eq!(self.len(), rhs.len());
        VecArray(
            self.iter()
                .zip(rhs.iter())
                .map(|(x, y)| x.clone() + y.clone())
                .collect(),
        )
    }
}

impl<T: Clone + Sub<Output = T>> Sub<VecArray<T>> for VecArray<T> {
    type Output = VecArray<T>;

    fn sub(self, rhs: VecArray<T>) -> VecArray<T> {
        assert_eq!(self.len(), rhs.len());
        VecArray(
            self.iter()
                .zip(rhs.iter())
                .map(|(x, y)| x.clone() - y.clone())
                .collect(),
        )
    }
}

impl NaturalArray<VecKind> for VecArray<usize> {
    fn max(&self) -> Option<usize> {
        self.iter().max().copied()
    }

    /// ```rust
    /// # use open_hypergraphs::array::{*, vec::*};
    /// let x = VecArray(vec![0, 1, 2, 3, 4, 5]);
    /// let d = 3;
    /// let expected_q = VecArray(vec![0, 0, 0, 1, 1, 1]);
    /// let expected_r = VecArray(vec![0, 1, 2, 0, 1, 2]);
    /// let (q, r) = x.quot_rem(d);
    /// assert_eq!(expected_q, q);
    /// assert_eq!(expected_r, r);
    /// ```
    fn quot_rem(&self, d: usize) -> (Self, Self) {
        assert!(d != 0);
        let mut q = Vec::with_capacity(self.len());
        let mut r = Vec::with_capacity(self.len());
        for x in self.iter() {
            q.push(x / d);
            r.push(x % d);
        }
        (VecArray(q), VecArray(r))
    }

    fn mul_constant_add(&self, c: usize, x: &Self) -> Self {
        assert_eq!(self.len(), x.len());
        let mut r = Vec::with_capacity(self.len());
        for (s, x) in self.iter().zip(x.iter()) {
            r.push(s * c + x)
        }
        VecArray(r)
    }

    /// ```rust
    /// # use open_hypergraphs::array::{*, vec::*};
    /// let input = VecArray(vec![1, 2, 3, 4]);
    /// let expected = VecArray(vec![0, 1, 3, 6, 10]);
    ///
    /// assert_eq!(input.cumulative_sum(), expected);
    /// ```
    fn cumulative_sum(&self) -> Self {
        let mut v = Vec::with_capacity(self.len() + 1);
        let mut a = 0;
        for x in self.iter() {
            v.push(a);
            a += x;
        }
        v.push(a); // don't forget the total sum!
        VecArray(v)
    }

    fn arange(start: &usize, stop: &usize) -> Self {
        assert!(stop >= start, "invalid range [{:?}, {:?})", start, stop);
        let n = stop - start;
        let mut v = Vec::with_capacity(n);
        for i in 0..n {
            v.push(start + i);
        }
        VecArray(v)
    }

    /// ```rust
    /// # use open_hypergraphs::array::*;
    /// # use open_hypergraphs::array::vec::*;
    /// let repeats: VecArray<usize> = VecArray(vec![1, 2, 0, 3]);
    /// let values: &[usize] = &[5, 6, 7, 8];
    /// let actual = repeats.repeat(values);
    /// let expected = VecArray::<usize>(vec![5, 6, 6, 8, 8, 8]);
    /// assert_eq!(actual, expected);
    /// ```
    fn repeat(&self, x: &[usize]) -> VecArray<usize> {
        assert_eq!(self.len(), x.len());
        let mut v: Vec<usize> = Vec::new();
        for (k, xi) in self.iter().zip(x) {
            v.extend(std::iter::repeat(xi).take(*k))
        }
        VecArray(v)
    }
}
