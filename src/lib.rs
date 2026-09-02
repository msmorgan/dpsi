// The README is the crate-level documentation, so its examples are doctests
// and the two texts can never drift apart.
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::HashSet;
use std::sync::LazyLock;
use std::sync::RwLock;

mod by_addr;
pub use by_addr::ByAddr;

/// The process-wide pool of interned strings.
static POOL: LazyLock<RwLock<HashSet<&'static str>>> = LazyLock::new(RwLock::default);

/// Look `s` up under a shared read lock without ever inserting it.
fn lookup(s: &str) -> Option<&'static str> {
    POOL.read()
        .expect("the ident pool is poisoned")
        .get(s)
        .copied()
}

/// Return the canonical `&'static str` for `s`, interning it if it is new.
fn intern(s: &str) -> &'static str {
    // Fast path: shared read lock for the overwhelmingly common hit.
    if let Some(interned) = lookup(s) {
        return interned;
    }
    let mut pool = POOL.write().expect("the ident pool is poisoned");
    // Another thread may have interned `s` between the two locks.
    pool.get(s).copied().unwrap_or_else(|| {
        let interned: &'static str = Box::leak(s.into());
        pool.insert(interned);
        interned
    })
}

/// An interned string with cheap copy and equality operations.
///
/// Every `Ident` is produced by the interner, so equal text always yields the
/// same pointer. That is the type's central invariant: `==` compares
/// addresses, not bytes, and is therefore O(1) no matter how long the string
/// is. Ordering deliberately does *not* follow the pointers — [`Ord`] and
/// [`PartialOrd`] compare the text, so sorted output is stable across runs.
///
/// Interning leaks. The first `Ident` for a given piece of text allocates a
/// `Box<str>` and leaks it, so the allocation lives for the rest of the
/// process and the pool only ever grows. That is the whole trade: it is what
/// buys the `'static` lifetime, the [`Copy`] implementation, and the pointer
/// equality. Intern identifiers and other bounded vocabularies; do not intern
/// unbounded input such as user-supplied text or file contents. To probe the
/// pool with untrusted text without growing it, use [`Ident::get`].
///
/// ```
/// use dpsi::Ident;
///
/// let from_literal = Ident::new("Dipsy");
/// let from_owned = Ident::new(&String::from("Dipsy"));
///
/// assert_eq!(from_literal, from_owned);
/// assert!(std::ptr::eq(from_literal.as_str(), from_owned.as_str()));
/// ```
#[expect(
    clippy::derived_hash_with_manual_eq,
    reason = "pointer-equal implies content-equal, so Eq implies the same hash"
)]
#[derive(Debug, Clone, Copy, Eq, Hash)]
pub struct Ident(&'static str);

impl Ident {
    /// Intern `s` and return its canonical identifier.
    ///
    /// If `s` has not been interned before, this leaks a copy of it that
    /// lives for the rest of the process.
    ///
    /// # Panics
    ///
    /// Panics if the interner's lock has been poisoned by a panic in another
    /// thread.
    #[must_use]
    pub fn new(s: &str) -> Self {
        Self(intern(s))
    }

    /// Return the existing identifier for `s`, or `None` if it has never been
    /// interned.
    ///
    /// Unlike [`Ident::new`], this never inserts and never leaks, so it is
    /// the safe way to probe the pool with untrusted text.
    ///
    /// # Panics
    ///
    /// Panics if the interner's lock has been poisoned by a panic in another
    /// thread.
    #[must_use]
    pub fn get(s: &str) -> Option<Self> {
        lookup(s).map(Self)
    }

    /// Return the interned string.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

// Every constructor goes through `intern`, so pointer equality is equivalent
// to content equality. The derived Hash remains content-based, preserving the
// `Borrow<str>` lookup contract.
impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

// Order by content, never by address: addresses depend on interning order and
// on the allocator, so a derived `Ord` would shuffle between runs.
impl Ord for Ident {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Route the empty string through the interner too, preserving the canonical
// pointer invariant.
impl Default for Ident {
    fn default() -> Self {
        Self::new("")
    }
}

impl From<&str> for Ident {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl std::ops::Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::borrow::Borrow<str> for Ident {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for Ident {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Ident {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

#[cfg(feature = "serde")]
mod serde_impls;
#[cfg(feature = "serde")]
pub use serde_impls::IdentSeed;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Ident;
    use super::POOL;

    #[test]
    fn equal_text_shares_one_allocation() {
        let first = Ident::new("Dipsy");
        let owned = String::from("Dipsy");
        let second = Ident::new(&owned);

        assert_eq!(first, second);
        assert!(std::ptr::eq(first.as_str(), second.as_str()));
    }

    #[test]
    fn interning_is_thread_safe() {
        let interns: Vec<_> = (0..8)
            .map(|_| std::thread::spawn(|| Ident::new("DumbestPossibleStringInterner")))
            .map(|thread| thread.join().unwrap())
            .collect();

        assert!(
            interns
                .windows(2)
                .all(|pair| std::ptr::eq(pair[0].as_str(), pair[1].as_str()))
        );
    }

    #[test]
    fn ord_follows_content_not_interning_order() {
        // Intern in reverse content order so that address order, whatever it
        // is, cannot accidentally agree with the expected result.
        let zebra = Ident::new("ord-zebra");
        let mango = Ident::new("ord-mango");
        let apple = Ident::new("ord-apple");

        let mut sorted = vec![zebra, mango, apple];
        sorted.sort();

        assert_eq!(sorted, vec![apple, mango, zebra]);
        assert!(apple < mango);
        assert!(mango < zebra);
        assert_eq!(apple.cmp(&apple), std::cmp::Ordering::Equal);
    }

    #[test]
    fn get_does_not_intern() {
        let absent = "get-never-interned-text";

        assert_eq!(Ident::get(absent), None);
        assert!(!POOL.read().unwrap().contains(absent));

        let present = Ident::new("get-interned-text");
        assert_eq!(Ident::get("get-interned-text"), Some(present));
    }

    #[test]
    fn default_is_the_interned_empty_string() {
        let default = Ident::default();

        assert!(std::ptr::eq(default.as_str(), Ident::new("").as_str()));
        assert_eq!(default, "");
    }

    #[test]
    fn hash_map_lookup_by_str_works() {
        let mut map = HashMap::new();
        map.insert(Ident::new("hash-map-key"), 42_u32);

        assert_eq!(map.get("hash-map-key"), Some(&42));
        assert_eq!(map.get("hash-map-missing"), None);
    }
}
