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

use core::time::Duration;

use kyron::{
    futures::sleep,
    time::clock::{Clock, Instant},
};
use kyron_foundation::prelude::warn;

use crate::events::event_traits::ListenerTrait;

pub(crate) struct TimerEvent {
    start_time: Option<Instant>,
    cycle_duration: core::time::Duration,
    tick: i128,
}

impl TimerEvent {
    pub fn new(cycle_duration: core::time::Duration) -> Self {
        TimerEvent {
            start_time: None,
            cycle_duration,
            tick: -1,
        }
    }
}

impl ListenerTrait for TimerEvent {
    fn next(&mut self) -> impl core::future::Future<Output = crate::prelude::ActionResult> + Send + 'static {
        // TODO: Fix the issue that next() i called before iteration to create future
        self.tick += 1;

        let is_first_time = self.start_time.is_none();
        if is_first_time {
            self.start_time = Some(Clock::now());
        }

        let start_time = self.start_time.unwrap();
        let cycle = self.cycle_duration;
        let tick = self.tick;

        // TODO: fix when mio is providing timer events, currently we use sleep
        async move {
            let elapsed = Clock::now().saturating_duration_since(start_time).as_millis();
            let elapsed_in_full_cycles = cycle.as_millis() * tick as u128;

            match elapsed.cmp(&elapsed_in_full_cycles) {
                core::cmp::Ordering::Less => {
                    let remaining = elapsed_in_full_cycles - elapsed;
                    let remaining_duration = Duration::from_millis(remaining as u64);
                    sleep::sleep(remaining_duration).await;
                },
                core::cmp::Ordering::Equal => {},
                core::cmp::Ordering::Greater => {
                    warn!(
                        "TimerEvent: cycle duration exceeded, expected cycle: {}, elapsed: {} in iteration {}",
                        cycle.as_millis(),
                        elapsed,
                        tick
                    );
                },
            }

            Ok(())
        }
    }
}
