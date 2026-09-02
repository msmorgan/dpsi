//! [`ByAddr`], an [`Ident`] wrapper whose hash is its address.

use crate::Ident;

/// An [`Ident`] that hashes by address instead of by content.
///
/// Reach for this when a map keyed by identifiers is hot and hashing the
/// bytes is what the lookup actually costs. [`Ident`]'s own [`Hash`] reads
/// the whole string on every probe, which keeps `Borrow<str>` working but
/// pays for the text again and again; `ByAddr` hashes the interned
/// allocation's address instead, so the work is constant no matter how long
/// the string is. Be honest about the size of the win: for the short
/// identifiers this crate is built for, hashing a handful of bytes is
/// already cheap, so the gain is modest — and, as of this writing,
/// unmeasured. A later benchmark ticket will say whether it is worth
/// reaching for at all.
///
/// What you give up is the `Borrow<str>` shortcut. A `HashMap<Ident, _>` can
/// be probed with a plain `&str`; a `HashMap<ByAddr, _>` cannot, because the
/// key's hash is an address and a `&str` does not have the *interned*
/// address. Build the key first, and build it with [`Ident::get`] rather
/// than [`Ident::new`]: `Ident::get(text).map(ByAddr)` returns `None` for
/// text that was never interned, whereas `Ident::new` would intern it —
/// leaking an allocation for every distinct miss, which is exactly the wrong
/// thing to do with untrusted input.
///
/// Iteration order of a `ByAddr`-keyed map is not reproducible across runs,
/// even under a deterministic hasher such as [Fx]. The hasher is only half
/// of it: the input is an address, and addresses depend on allocation order
/// and on the allocator, so they differ from one run to the next. Sort the
/// entries — [`Ord`] on `ByAddr` compares text, same as on [`Ident`] — if
/// you need stable output.
///
/// The addresses themselves are well behaved as hasher input with one
/// caveat: they are aligned, so their low bits are always zero. Any hasher
/// that mixes — `SipHash`, which [`std`] uses by default, or Fx — spreads
/// them fine. An *identity* hasher such as [`nohash`], which passes the
/// value through untouched, will pile every key into a fraction of the
/// buckets. Do not pair this type with one.
///
/// ```
/// use std::collections::HashMap;
///
/// use dpsi::ByAddr;
/// use dpsi::Ident;
///
/// let mut arity: HashMap<ByAddr, u32> = HashMap::new();
/// arity.insert(ByAddr(Ident::new("by-addr-doc-cons")), 2);
///
/// // Probe with `Ident::get`, which returns `None` instead of interning.
/// let hit = Ident::get("by-addr-doc-cons").map(ByAddr);
/// assert_eq!(hit.and_then(|key| arity.get(&key)), Some(&2));
///
/// assert!(Ident::get("by-addr-doc-never-seen").is_none());
/// ```
///
/// [Fx]: https://crates.io/crates/rustc-hash
/// [`nohash`]: https://crates.io/crates/nohash
#[derive(Clone, Copy, Default)]
pub struct ByAddr(
    /// The wrapped identifier.
    pub Ident,
);

// Print as `ByAddr("text")` rather than `ByAddr(Ident("text"))`: the wrapper
// is a key-strategy marker, and the nesting says nothing a reader wants.
impl std::fmt::Debug for ByAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("ByAddr").field(&self.0.as_str()).finish()
    }
}

// The whole point of the type: hash the interned allocation's address, which
// is O(1), instead of the bytes behind it. `<*const u8>::addr()` is the
// pointer-to-integer cast written without an `as`, so no lint fires and no
// `expect` is needed.
impl std::hash::Hash for ByAddr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_str().as_ptr().addr().hash(state);
    }
}

// Delegated to `Ident`, so this is the same pointer comparison. Equal text
// interns to one address, so pointer-equal keys hash alike and the `Hash`
// contract holds.
impl PartialEq for ByAddr {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ByAddr {}

// Order by content, like `Ident`: addresses vary between runs, and sorting is
// the way to get reproducible output out of an address-keyed map.
impl Ord for ByAddr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for ByAddr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::ops::Deref for ByAddr {
    type Target = Ident;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Ident> for ByAddr {
    fn from(ident: Ident) -> Self {
        Self(ident)
    }
}

impl From<ByAddr> for Ident {
    fn from(by_addr: ByAddr) -> Self {
        by_addr.0
    }
}

impl std::fmt::Display for ByAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// Transparent on the wire: a `ByAddr` is indistinguishable from the `Ident`
// it wraps, which is itself indistinguishable from its text.
#[cfg(feature = "serde")]
impl serde::Serialize for ByAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ByAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ident::deserialize(deserializer).map(ByAddr)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::hash::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    use super::ByAddr;
    use crate::Ident;

    /// Hash `value` on its own with the standard default hasher.
    fn hash_of(value: &ByAddr) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn hash_map_round_trips_and_misses_uninterned_text() {
        let mut map = HashMap::new();
        map.insert(ByAddr(Ident::new("by-addr-map-key")), 42_u32);

        let hit = Ident::get("by-addr-map-key").map(ByAddr);
        assert_eq!(hit.and_then(|key| map.get(&key)), Some(&42));

        // Text that was never interned has no key to probe with at all, which
        // is the miss `Ident::get` is there to give without interning it.
        assert_eq!(Ident::get("by-addr-map-never-interned"), None);
        assert!(!map.contains_key(&ByAddr(Ident::new("by-addr-map-other"))));
    }

    #[test]
    fn equal_text_hashes_equally() {
        let first = ByAddr(Ident::new("by-addr-hash-eq"));
        let owned = String::from("by-addr-hash-eq");
        let second = ByAddr(Ident::new(&owned));

        assert_eq!(first, second);
        assert_eq!(hash_of(&first), hash_of(&second));
    }

    #[test]
    fn ord_matches_ident() {
        // Interned in reverse content order, so address order cannot
        // accidentally agree with the expected result.
        let zebra = Ident::new("by-addr-ord-zebra");
        let mango = Ident::new("by-addr-ord-mango");
        let apple = Ident::new("by-addr-ord-apple");

        for pair in [
            (apple, mango),
            (mango, zebra),
            (zebra, apple),
            (apple, apple),
        ] {
            let (left, right) = pair;
            assert_eq!(ByAddr(left).cmp(&ByAddr(right)), left.cmp(&right));
        }

        let mut sorted = vec![ByAddr(zebra), ByAddr(mango), ByAddr(apple)];
        sorted.sort();
        assert_eq!(sorted, vec![ByAddr(apple), ByAddr(mango), ByAddr(zebra)]);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_round_trip_keeps_text_and_pointer() {
        let original = ByAddr(Ident::new("by-addr-serde-round-trip"));

        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, r#""by-addr-serde-round-trip""#);

        let back: ByAddr = serde_json::from_str(&json).unwrap();
        assert_eq!(back.as_str(), "by-addr-serde-round-trip");
        assert!(std::ptr::eq(back.0.as_str(), original.0.as_str()));
        assert_eq!(back, original);
    }
}
