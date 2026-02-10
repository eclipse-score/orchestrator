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

use ::core::time::Duration;

use crate::common::tag::Tag;

use core::fmt::Debug;

pub trait MeterTrait {
    fn new(id: Tag) -> Self;
    fn reset(&mut self);
    fn meter<T: Debug>(&mut self, duration: &Duration, info: T);
}

pub struct NoneMeter {}
impl MeterTrait for NoneMeter {
    fn new(_id: Tag) -> Self {
        Self {}
    }

    fn reset(&mut self) {}

    fn meter<T: Debug>(&mut self, _duration: &Duration, _info: T) {}
}

pub struct Meter<const PRINT_INTERVAL: u32 = 1000> {
    id: Tag,
    running_average: i64,
    cnt: u32,
}

impl<const PRINT_INTERVAL: u32> Meter<PRINT_INTERVAL> {
    fn meter_impl(&mut self, iter_duration: &Duration) {
        self.cnt += 1;
        self.running_average += (iter_duration.as_micros() as i64 - self.running_average) / (self.cnt as i64);
    }

    fn reset_impl(&mut self) {
        self.cnt = 0;
        self.running_average = 0;
    }

    fn new(id: Tag) -> Self {
        Self {
            id,
            cnt: 0,
            running_average: 0,
        }
    }
}

impl<const PRINT_INTERVAL: u32> MeterTrait for Meter<PRINT_INTERVAL> {
    fn new(id: Tag) -> Self {
        Self::new(id)
    }

    fn reset(&mut self) {
        self.reset_impl();
    }

    fn meter<T: Debug>(&mut self, duration: &Duration, info: T) {
        if self.cnt == PRINT_INTERVAL {
            println!(
                "{:?}: Iteration took {}us, additional info: {:?}",
                self.id, self.running_average, info
            );
            self.reset_impl();
        }

        self.meter_impl(duration);
    }
}
