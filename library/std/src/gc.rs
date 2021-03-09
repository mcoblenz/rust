#![allow(missing_docs)]
#![cfg(not(bootstrap))]


use crate::boxed::Box;
use crate::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use crate::vec::Vec;
use crate::hash::{BuildHasher, Hash};
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use crate::path::{Path, PathBuf};
use crate::rc::Rc;
use core::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
    AtomicU64, AtomicU8, AtomicUsize,
};
use crate::cell::Cell;
use crate::borrow::Borrow;


/// The Finalize trait, which needs to be implemented on
/// garbage-collected objects to define finalization logic.
#[unstable(
    feature = "bronze_gc",
    issue = "none",
    reason = "GC is experimental"
)]
#[lang = "gcfinalize"]
pub trait Finalize {
    /// Does any user-defined cleanup needed before the object is deallocated.
    fn finalize(&self) {}
}

#[unstable(
    feature = "bronze_gc",
    issue = "none",
    reason = "GC is experimental"
)]
// #[cfg_attr(not(test), rustc_diagnostic_item = "gctrace_trait")]
#[lang = "gctrace"]
// TODO: make this an auto trait
/// The GcTrace trait is required for traceability by the garbage collector.
pub unsafe trait GcTrace : Finalize {
    /// Traces references for garbage collection.
    unsafe fn trace(&self);

    /// Runs Finalize::finalize() on this object and all
    /// contained subobjects
    fn finalize_glue(&self);
}

// From Manishearth's rust-gc.


/// This rule implements the trace methods with empty implementations.
///
/// Use this for marking types as not containing any `GcTrace` types.
#[unstable(
    feature = "bronze_gc",
    issue = "none",
    reason = "GC is experimental"
)]
#[macro_export]
macro_rules! unsafe_empty_trace {
    () => {
        #[inline]
        unsafe fn trace(&self) {}
        #[inline]
        fn finalize_glue(&self) {
            $crate::gc::Finalize::finalize(self)
        }
    };
}


/// This rule implements the trace method.
///
/// You define a `this` parameter name and pass in a body, which should call `mark` on every
/// traceable element inside the body. The mark implementation will automatically delegate to the
/// correct method on the argument.
#[unstable(
    feature = "bronze_gc",
    issue = "none",
    reason = "GC is experimental"
)]
#[macro_export]
macro_rules! custom_trace {
    ($this:ident, $body:expr) => {
        #[inline]
        unsafe fn trace(&self) {
            #[inline]
            unsafe fn mark<T: $crate::gc::GcTrace + ?Sized>(it: &T) {
                $crate::gc::GcTrace::trace(it);
            }
            let $this = self;
            $body
        }
        #[inline]
        fn finalize_glue(&self) {
            $crate::gc::Finalize::finalize(self);
            #[inline]
            fn mark<T: $crate::gc::GcTrace + ?Sized>(it: &T) {
                $crate::gc::GcTrace::finalize_glue(it);
            }
            let $this = self;
            $body
        }
    };
}

/// Creates an empty finalize and trace implementation.
/// Appropriate for types that have no fields that need to be traced.
#[unstable(
    feature = "bronze_gc",
    issue = "none",
    reason = "GC is experimental"
)]
#[macro_export]
macro_rules! simple_empty_finalize_trace {
    ($($T:ty),*) => {
        $(
            impl Finalize for $T {}
            unsafe impl GcTrace for $T { unsafe_empty_trace!(); }
        )*
    }
}

#[unstable(
    feature = "bronze_gc",
    issue = "none",
    reason = "GC is experimental"
)]
simple_empty_finalize_trace![
    (),
    bool,
    isize,
    usize,
    i8,
    u8,
    i16,
    u16,
    i32,
    u32,
    i64,
    u64,
    i128,
    u128,
    f32,
    f64,
    char,
    String,
    Box<str>,
    Rc<str>,
    Path,
    PathBuf,
    NonZeroIsize,
    NonZeroUsize,
    NonZeroI8,
    NonZeroU8,
    NonZeroI16,
    NonZeroU16,
    NonZeroI32,
    NonZeroU32,
    NonZeroI64,
    NonZeroU64,
    NonZeroI128,
    NonZeroU128,
    AtomicBool,
    AtomicIsize,
    AtomicUsize,
    AtomicI8,
    AtomicU8,
    AtomicI16,
    AtomicU16,
    AtomicI32,
    AtomicU32,
    AtomicI64,
    AtomicU64
];


impl<T: GcTrace> Finalize for Option<T> {}
unsafe impl<T: GcTrace> GcTrace for Option<T> {
    custom_trace!(this, {
        if let Some(ref v) = *this {
            mark(v);
        }
    });
}

impl<T: GcTrace> Finalize for Vec<T> {
    fn finalize(&self) {
    }
}
unsafe impl<T: GcTrace> GcTrace for Vec<T> {
    custom_trace!(this, {

        for v in this.iter() {
            mark(v);
        }
    });
}

impl<T: GcTrace + ?Sized> Finalize for Box<T> {}
unsafe impl<T: GcTrace + ?Sized> GcTrace for Box<T> {
    custom_trace!(this, {
        mark(&**this);
    });
}

impl<T: GcTrace> Finalize for Box<[T]> {}
unsafe impl<T: GcTrace> GcTrace for Box<[T]> {
    custom_trace!(this, {
        for e in this.iter() {
            mark(e);
        }
    });
}

impl<T: GcTrace> Finalize for Rc<T> {}
unsafe impl<T: GcTrace> GcTrace for Rc<T> {
    custom_trace!(this, {
        mark::<T>(this.borrow());
    });
}

impl<T: GcTrace + Copy> Finalize for Cell<T> {}
unsafe impl<T: GcTrace + Copy> GcTrace for Cell<T> {
    custom_trace!(this, {
        mark(&this.get());
    });
}

impl<T: GcTrace, E: GcTrace> Finalize for Result<T, E> {}
unsafe impl<T: GcTrace, E: GcTrace> GcTrace for Result<T, E> {
    custom_trace!(this, {
        match *this {
            Ok(ref v) => mark(v),
            Err(ref v) => mark(v),
        }
    });
}


impl<T: Ord + GcTrace> Finalize for BinaryHeap<T> {}
unsafe impl<T: Ord + GcTrace> GcTrace for BinaryHeap<T> {
    custom_trace!(this, {
        for v in this.iter() {
            mark(v);
        }
    });
}

impl<K: GcTrace, V: GcTrace> Finalize for BTreeMap<K, V> {}
unsafe impl<K: GcTrace, V: GcTrace> GcTrace for BTreeMap<K, V> {
    custom_trace!(this, {
        for (k, v) in this {
            mark(k);
            mark(v);
        }
    });
}

impl<T: GcTrace> Finalize for BTreeSet<T> {}
unsafe impl<T: GcTrace> GcTrace for BTreeSet<T> {
    custom_trace!(this, {
        for v in this {
            mark(v);
        }
    });
}

impl<K: Eq + Hash + GcTrace, V: GcTrace, S: BuildHasher> Finalize for HashMap<K, V, S> {}
unsafe impl<K: Eq + Hash + GcTrace, V: GcTrace, S: BuildHasher> GcTrace for HashMap<K, V, S> {
    custom_trace!(this, {
        for (k, v) in this.iter() {
            mark(k);
            mark(v);
        }
    });
}

impl<T: Eq + Hash + GcTrace, S: BuildHasher> Finalize for HashSet<T, S> {}
unsafe impl<T: Eq + Hash + GcTrace, S: BuildHasher> GcTrace for HashSet<T, S> {
    custom_trace!(this, {
        for v in this.iter() {
            mark(v);
        }
    });
}

impl<T: Eq + Hash + GcTrace> Finalize for LinkedList<T> {}
unsafe impl<T: Eq + Hash + GcTrace> GcTrace for LinkedList<T> {
    custom_trace!(this, {
        for v in this.iter() {
            mark(v);
        }
    });
}

impl<T: GcTrace> Finalize for VecDeque<T> {}
unsafe impl<T: GcTrace> GcTrace for VecDeque<T> {
    custom_trace!(this, {
        for v in this.iter() {
            mark(v);
        }
    });
}
