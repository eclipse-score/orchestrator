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

use super::tag::Tag;
use crate::program_database::ActionProvider;
use ::core::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
/// OrchTagId is a struct that contains the tag ID, a key for SlotMap, and a SlotMap identifier.
pub(crate) struct OrchTagId {
    tag: Tag,
}

/// OrchestrationTag is a wrapper around OrchTagId that provides a convenient way to create and manage orchestration tags.
/// It contains tag ID, a key for SlotMap, and a SlotMap identifier.
/// The tag ID is used to uniquely identify the orchestration tag.
#[derive(Debug, Clone)]
pub struct OrchestrationTag {
    id: OrchTagId,
    action_provider: Rc<RefCell<ActionProvider>>,
}

#[allow(dead_code)]
impl OrchestrationTag {
    /// Create a new orchestration tag with the given Tag.
    pub(crate) fn new(tag: Tag, action_provider: Rc<RefCell<ActionProvider>>) -> Self {
        Self {
            id: OrchTagId { tag },
            action_provider,
        }
    }

    /// Get the tag ID of the orchestration tag.
    #[inline]
    pub(crate) fn tag(&self) -> &Tag {
        &self.id.tag
    }

    #[inline]
    pub(crate) fn action_provider(&self) -> &RefCell<ActionProvider> {
        self.action_provider.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::common::DesignConfig;

    use super::*;
    use ::core::cell::RefCell;

    #[test]
    fn orchestration_tag_creation() {
        let config = DesignConfig::default();
        let ap = Rc::new(RefCell::new(ActionProvider::new(config)));
        let tag = OrchestrationTag::new(Tag::from_str_static("test_tag"), ap.clone());
        assert_eq!(*tag.tag(), Tag::from_str_static("test_tag"));
    }
}
