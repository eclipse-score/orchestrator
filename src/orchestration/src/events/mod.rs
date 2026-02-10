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

pub mod event_traits;
pub mod events_provider;
pub mod local_events;
pub mod timer_events;

#[cfg(feature = "iceoryx2-ipc")]
pub(crate) mod iceoryx;
#[cfg(not(feature = "iceoryx2-ipc"))]
pub(crate) mod stub_global_events;

#[cfg(feature = "iceoryx2-ipc")]
pub type GlobalEventProvider = super::events::iceoryx::global_events::GlobalEvents;
#[cfg(not(feature = "iceoryx2-ipc"))]
pub type GlobalEventProvider = super::events::stub_global_events::StubGlobalEvents;
