use crate::array::*;
use crate::category::*;
use crate::finite_function::*;
use crate::semifinite::*;

use core::fmt::Debug;
use num_traits::{One, Zero};

// The minimum set of operations some arrows must have in order to define an [`IndexedCoproduct`]
// over them.
pub trait HasLen<K: ArrayKind> {
    fn len(&self) -> K::I;
    fn is_empty(&self) -> bool {
        self.len() == K::I::zero()
    }
}

impl<K: ArrayKind> HasLen<K> for FiniteFunction<K> {
    fn len(&self) -> K::I {
        self.source()
    }
}

impl<K: ArrayKind, T> HasLen<K> for SemifiniteFunction<K, T>
where
    K::Type<T>: Array<K, T>,
{
    fn len(&self) -> K::I {
        self.0.len()
    }
}

// TODO: replace sources with a FiniteFunction<K> of *pointers* whose codomain is total size?
// This lets us remove a lot of trait bounds.
/// A finite coproduct of arrows of type `A`.
/// Pragmatically, it's a segmented array
#[non_exhaustive] // force construction via new.
pub struct IndexedCoproduct<K: ArrayKind, F> {
    /// A ['FiniteFunction'] satisfying `self.target() = self.table.sum() + 1`
    pub sources: FiniteFunction<K>,

    /// The concatenation of all arrays in the coproduct.
    pub values: F,
}

impl<K: ArrayKind, F: Clone> Clone for IndexedCoproduct<K, F>
where
    K::Type<K::I>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            sources: self.sources.clone(),
            values: self.values.clone(),
        }
    }
}

impl<K: ArrayKind, F: Clone + HasLen<K>> IndexedCoproduct<K, F>
where
    K::Type<K::I>: NaturalArray<K>,
{
    /// Create a new IndexedCoproduct from a FiniteFunction whose target is the sum of its
    /// elements. This condition is checked by summing the array.
    pub fn new(sources: FiniteFunction<K>, values: F) -> Option<Self> {
        // use the from_semifinite construct, but check against declared sum anyway.
        let target = sources.target();
        let result = Self::from_semifinite(SemifiniteFunction(sources.table.into()), values)?;
        if result.sources.target() != target {
            return None;
        }

        Some(result)
    }

    pub fn from_semifinite(sources: SemifiniteFunction<K, K::I>, values: F) -> Option<Self> {
        let sum = sources.0.as_ref().sum();
        if sum != values.len() {
            return None;
        }

        let sources = FiniteFunction::new(sources.0.into(), sum + K::I::one()).unwrap();
        Some(IndexedCoproduct { sources, values })
    }

    pub fn singleton(values: F) -> Self {
        let n = values.len();
        let sources = FiniteFunction::constant(K::I::one(), n, K::I::zero());
        IndexedCoproduct { sources, values }
    }

    pub fn len(&self) -> K::I {
        self.sources.source()
    }

    /// Compose two `IndexedCoproduct` thought of as lists-of-lists.
    ///
    /// An indexed (finite) coproduct `c` consists of a mapping
    /// `s : A → K`
    /// and
    /// of arrows `f : s(a) is a map `x : Σ_{a ∈ A} s(a) → B`,
    /// where `s(a)
    ///
    ///
    /// ```text
    /// x : Σ_{a ∈ A} s(a) → B      aka A → B*
    /// y : Σ_{b ∈ B} s(b) → C      aka B → C*
    /// z : Σ_{a ∈ A} s'(a) → C     aka A → C*
    /// ```
    pub fn flatmap(&self, other: &IndexedCoproduct<K, F>) -> IndexedCoproduct<K, F> {
        let sources = FiniteFunction {
            table: self.sources.table.segmented_sum(&other.sources.table),
            target: other.sources.target.clone(), // TODO: write a test for this
        };
        let values = other.values.clone();
        IndexedCoproduct { sources, values }
    }
}

impl<K: ArrayKind, F> HasLen<K> for IndexedCoproduct<K, F>
where
    K::Type<K::I>: NaturalArray<K>,
{
    fn len(&self) -> K::I {
        self.sources.len()
    }
}

// Special case methods where the values are finite functions.
impl<K: ArrayKind> IndexedCoproduct<K, FiniteFunction<K>>
where
    K::Type<K::I>: NaturalArray<K>,
{
    /// The initial object, i.e., the finite coproduct indexed by the empty set
    /// Note that the target of `sources` must be zero for laws to work here.
    pub fn initial(target: K::I) -> Self {
        let sources = FiniteFunction::initial(K::I::one());
        let values = FiniteFunction::initial(target);
        IndexedCoproduct { sources, values }
    }

    // This could generalise to any type with a tensor product, but we only need it for finite functions
    pub fn tensor(
        &self,
        other: &IndexedCoproduct<K, FiniteFunction<K>>,
    ) -> IndexedCoproduct<K, FiniteFunction<K>> {
        // build a new finite function for 'sources'. it consists of:
        //  - concatenated segment sizes
        //  - target equal to *total sum* (sum of targets)
        let table = self.sources.table.concatenate(&other.sources.table);
        let target = (self.sources.target.clone() + other.sources.target.clone()) - K::I::one();

        IndexedCoproduct {
            sources: FiniteFunction { table, target },
            values: &self.values | &other.values,
        }
    }

    pub fn indexed_values(&self, _x: &FiniteFunction<K>) -> FiniteFunction<K> {
        todo!()
    }

    /// Map the *values* array of an indexed coproduct, leaving the sources unchanged.
    ///
    /// Given an indexed coproduct
    ///
    /// ```text
    /// Σ_{i ∈ I} f_i : Σ_{i ∈ I} A_i → B
    /// ```
    ///
    /// and a finite function `x : B → C`,
    /// return a new [`IndexedCoproduct`] representing
    ///
    /// ```text
    /// Σ_{i ∈ I} (f_i ; x) : Σ_{i ∈ I} A_i → C
    /// ```
    ///
    /// Returns `None` if `x.source() != B`.
    pub fn map_values(&self, x: &FiniteFunction<K>) -> Option<Self> {
        Some(Self {
            sources: self.sources.clone(),
            values: (&self.values >> x)?,
        })
    }

    pub fn map_indexes(&self, _x: &FiniteFunction<K>) -> Self {
        todo!()
    }
}

impl<K: ArrayKind, F: Debug> Debug for IndexedCoproduct<K, F>
where
    K::Index: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexedCoproduct")
            .field("sources", &self.sources)
            .field("values", &self.values)
            .finish()
    }
}
