// *******************************************************************************
// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
// *******************************************************************************

use ::core::fmt::{Debug, Formatter};

///
/// Tag implementation with an 'id' and 'tracing info string'. Supports creation of Tag from &str and String, also from_str_ref().
///
#[derive(Clone, Copy)]
pub struct Tag {
    id: u64,
    tracing_str: &'static str,
}

impl Eq for Tag {}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Tag {
    /// Create Tag from static string.
    pub fn from_str_static(s: &'static str) -> Self {
        // This do not leak anything so we don't need to keep it in registry
        Self {
            id: Tag::compute_djb2_hash(s),
            tracing_str: s,
        }
    }

    /// ID of the Tag.
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Tracing info string of the Tag. It is empty if 'orch_tracing' is not enabled during compilation.
    #[inline]
    pub fn tracing_str(&self) -> &str {
        self.tracing_str
    }

    // Function to calculate hash of string to use it as Tag id.
    const fn compute_djb2_hash(s: &str) -> u64 {
        let bytes = s.as_bytes();
        let mut hash: u64 = 5381;
        let mut i = 0;

        while i < bytes.len() {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(bytes[i] as u64); // hash * 33 + c
            i += 1;
        }

        hash
    }

    /// Find Tag in a collection of items where you cannot do Key -> value mapping where Tag would be a key
    pub fn find_in_collection<T: AsTagTrait, C: Iterator<Item = T>>(&self, mut c: C) -> Option<T> {
        c.find(|e| e.as_tag() == self)
    }

    /// Checks if Tag is in a collection of items where you cannot do Key -> value mapping where Tag would be a key
    pub fn is_in_collection<T: AsTagTrait, C: Iterator<Item = T>>(&self, mut c: C) -> bool {
        c.any(|e| e.as_tag() == self)
    }
}

/// Create Tag from &str.
#[allow(clippy::from_over_into)]
impl Into<Tag> for &str {
    #[cfg(feature = "orch_tracing")]
    fn into(self) -> Tag {
        let mut r = internal::TAG_REGISTRY.lock().unwrap();
        let id = Tag::compute_djb2_hash(self);

        r.get(id).unwrap_or_else(|| {
            r.insert_tag(Tag {
                id,
                tracing_str: self.to_owned().leak(),
            })
        })
    }

    #[cfg(not(feature = "orch_tracing"))]
    fn into(self) -> Tag {
        Tag {
            id: Tag::compute_djb2_hash(self),
            tracing_str: "",
        }
    }
}

/// Create Tag from String.
#[allow(clippy::from_over_into)]
impl Into<Tag> for String {
    #[cfg(feature = "orch_tracing")]
    fn into(self) -> Tag {
        let mut r = internal::TAG_REGISTRY.lock().unwrap();
        let id = Tag::compute_djb2_hash(&self);

        r.get(id).unwrap_or_else(|| {
            r.insert_tag(Tag {
                id,
                tracing_str: self.leak(),
            })
        })
    }

    #[cfg(not(feature = "orch_tracing"))]
    fn into(self) -> Tag {
        Tag {
            id: Tag::compute_djb2_hash(&self),
            tracing_str: "",
        }
    }
}

// Implementation of Debug fmt for Tag.
impl Debug for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "Tag(id:{}, str: {})", self.id, self.tracing_str)
    }
}

/// Trait to convert any type that implements `AsTagTrait` to a `Tag`. Helpful when storing custom types in collections that require search by `Tag`.
pub trait AsTagTrait {
    /// Convert self to Tag.
    fn as_tag(&self) -> &Tag;
}

#[cfg(feature = "orch_tracing")]
mod internal {
    // This is done to not leak strings that are used to build Tags if we already have them leaked once. We
    // hold reference in Tag to keep it lean and make it easy to disable in code if needed.

    use super::*;

    pub(super) static TAG_REGISTRY: std::sync::Mutex<TagRegistry> = std::sync::Mutex::new(TagRegistry::new());

    pub(super) struct TagRegistry {
        tags: Vec<Tag>,
    }

    impl TagRegistry {
        pub(super) const fn new() -> Self {
            Self { tags: Vec::new() }
        }

        pub(super) fn insert_tag(&mut self, value: Tag) -> Tag {
            match self.tags.binary_search(&value) {
                Ok(pos) => self.tags[pos],
                Err(pos) => {
                    self.tags.insert(pos, value);
                    self.tags[pos]
                },
            }
        }

        pub(super) fn get(&mut self, id: u64) -> Option<Tag> {
            let tag = Tag { id, tracing_str: "" };

            match self.tags.binary_search(&tag) {
                Ok(pos) => Some(self.tags[pos]),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tracing_str(ins: &str, _expected: &str) {
        #[cfg(feature = "orch_tracing")]
        assert_eq!(ins, _expected);
        #[cfg(not(feature = "orch_tracing"))]
        assert_eq!(ins, "");
    }

    #[test]
    fn test_tag_from_str() {
        let tag = Tag::from_str_static("test_info");
        let expected_id = Tag::compute_djb2_hash("test_info");

        assert_eq!(tag.id(), expected_id);
        assert_eq!(tag.tracing_str(), "test_info");
    }

    #[test]
    fn test_tag_from_static_str() {
        let tag: Tag = "static_tag".into();
        let expected_id = Tag::compute_djb2_hash("static_tag");

        assert_eq!(tag.id(), expected_id);
        assert_tracing_str(tag.tracing_str(), "static_tag");
    }

    #[test]
    fn test_tag_from_string() {
        let input = String::from("dynamic_tag");
        let expected_id = Tag::compute_djb2_hash(&input);

        let tag: Tag = input.clone().into();

        assert_eq!(tag.id(), expected_id);
        assert_tracing_str(tag.tracing_str(), "dynamic_tag");
    }

    #[test]
    fn test_compute_qorhash_known_value() {
        const EXPECTED_HASH: u64 = 229465095248369; // Precomputed for "example"
        let actual_hash = Tag::compute_djb2_hash("example");

        assert_eq!(actual_hash, EXPECTED_HASH);
    }

    #[test]
    fn test_consistent_hash_for_same_str() {
        let static_tag: Tag = "consistency".into();
        let string_tag: Tag = String::from("consistency").into();

        assert_eq!(static_tag.id, string_tag.id);
        assert_tracing_str(static_tag.tracing_str(), "consistency");
        assert_tracing_str(string_tag.tracing_str(), "consistency");
    }

    #[test]
    fn test_tag_comparision() {
        let tag1: Tag = "same_string".into();
        let tag2: Tag = String::from("same_string").into();
        let tag3: Tag = String::from("different_string").into();

        assert!(tag1 == tag2);
        assert!(tag1 != tag3);
    }
}
