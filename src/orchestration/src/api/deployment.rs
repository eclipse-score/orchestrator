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

use std::rc::Rc;

use crate::{
    api::{
        design::{Design, DesignTag},
        OrchestrationApi, _DesignTag,
    },
    common::tag::Tag,
    program::ProgramBuilder,
};
use kyron::core::types::UniqueWorkerId;
use kyron_foundation::prelude::CommonErrors;

pub struct Deployment<'a> {
    api: &'a mut OrchestrationApi<_DesignTag>,
}

impl Deployment<'_> {
    pub fn new(api: &mut OrchestrationApi<_DesignTag>) -> Deployment<'_> {
        Deployment { api }
    }

    /// Maps a system events to user events. This means that the specified user events will be treated as global events across all processes.
    pub fn bind_events_as_global(&mut self, system_event: &str, events_to_bind: &[Tag]) -> Result<(), CommonErrors> {
        let mut ret = Err(CommonErrors::NotFound);

        let creator = self.api.events.specify_global_event(system_event, events_to_bind)?;

        for d in &mut self.api.designs {
            // This logic allows to report NotFound only if no design has the event.
            ret =
                d.db.set_creator_for_events(Rc::clone(&creator), events_to_bind)
                    .or_else(|e| if e == CommonErrors::NotFound { ret } else { Err(e) })
        }

        ret
    }

    /// Binds user events to a local event. This means that the specified user events will be treated as local events within the process boundaries.
    pub fn bind_events_as_local(&mut self, events_to_bind: &[Tag]) -> Result<(), CommonErrors> {
        let mut ret = Err(CommonErrors::NotFound);

        let creator = self.api.events.specify_local_event(events_to_bind)?;

        for d in &mut self.api.designs {
            // This logic allows to report NotFound only if no design has the event.
            ret =
                d.db.set_creator_for_events(Rc::clone(&creator), events_to_bind)
                    .or_else(|e| if e == CommonErrors::NotFound { ret } else { Err(e) })
        }

        ret
    }

    /// Binds user events to a timer with given params
    pub fn bind_events_as_timer(
        &mut self,
        events_to_bind: &[Tag],
        cycle_duration: core::time::Duration,
    ) -> Result<(), CommonErrors> {
        let mut ret = Err(CommonErrors::NotFound);

        let creator = self.api.events.specify_timer_event(events_to_bind, cycle_duration)?;

        for d in &mut self.api.designs {
            // This logic allows to report NotFound only if no design has the event.
            ret =
                d.db.set_creator_for_events(Rc::clone(&creator), events_to_bind)
                    .or_else(|e| if e == CommonErrors::NotFound { ret } else { Err(e) })
        }

        ret
    }

    /// Binds an invoke action to a worker across all designs wherever that invoke action is registered.
    /// The registered invoke action will always be executed by the specified worker.
    /// # Arguments
    /// * `tag` - The tag of the invoke action to bind.
    /// * `worker_id` - The unique identifier of the worker to bind the invoke action to.
    ///
    pub fn bind_invoke_to_worker(&mut self, tag: Tag, worker_id: UniqueWorkerId) -> Result<(), CommonErrors> {
        let mut ret = Err(CommonErrors::NotFound);

        for d in &mut self.api.designs {
            // This logic allows to report NotFound only if no design has the event.
            ret = d.db.set_invoke_worker_id(tag, worker_id).or_else(|e| {
                if e == CommonErrors::NotFound {
                    ret
                } else {
                    Err(e)
                }
            })
        }

        ret
    }

    /// Binds a shutdown event as a global event.
    pub fn bind_shutdown_event_as_global(&mut self, system_event: &str, event: Tag) -> Result<(), CommonErrors> {
        let creator = self.api.events.specify_global_event(system_event, &[event])?;
        self.api.register_shutdown_event(event, creator)
    }

    /// Binds a shutdown event as a local event.
    pub fn bind_shutdown_event_as_local(&mut self, event: Tag) -> Result<(), CommonErrors> {
        let creator = self.api.events.specify_local_event(&[event])?;
        self.api.register_shutdown_event(event, creator)
    }

    /// Adds a program to the design. The program is created using the provided closure, which receives a mutable reference to the design.
    ///
    /// # Returns
    /// `Ok(())` if the program was added successfully
    /// `Err(CommonErrors::AlreadyDone)` if the design already has programs
    /// `Err(CommonErrors::NotFound)` if the design with the specified tag was not
    ///
    pub fn add_program<F>(&mut self, design_tag: DesignTag, program: F, name: &'static str) -> Result<(), CommonErrors>
    where
        F: FnOnce(&mut Design, &mut ProgramBuilder) -> Result<(), CommonErrors> + 'static,
    {
        let p = &mut self.api.designs.iter_mut().find(|d| d.id() == design_tag);

        if let Some(design) = p {
            if design.has_any_programs() {
                Err(CommonErrors::AlreadyDone)
            } else {
                design.add_program(name, Box::new(program));
                Ok(())
            }
        } else {
            Err(CommonErrors::NotFound)
        }
    }
}

#[cfg(test)]
#[cfg(not(miri))]
#[cfg(not(loom))]
mod tests {
    use super::*;
    use crate::common::DesignConfig;
    use core::marker::PhantomData;
    use kyron_foundation::containers::growable_vec::GrowableVec;

    fn setup_api_single_design() -> OrchestrationApi<crate::api::_DesignTag> {
        let design_tag = Tag::from_str_static("test_design");
        let params = DesignConfig::default();
        let design = crate::api::design::Design::new(design_tag, params);

        design.register_event("SomeUserEvent".into()).unwrap();

        let mut api = OrchestrationApi {
            designs: kyron_foundation::containers::growable_vec::GrowableVec::default(),
            events: crate::events::events_provider::EventsProvider::default(),
            shutdown_events: GrowableVec::default(),
            _p: PhantomData,
        };
        api.designs.push(design);
        api.design_done()
    }

    fn setup_api_multiple_design() -> OrchestrationApi<crate::api::_DesignTag> {
        let design_tag = Tag::from_str_static("test_design");
        let params = DesignConfig::default();
        let design = crate::api::design::Design::new(design_tag, params);

        design.register_event("SomeUserEvent".into()).unwrap();

        let mut api = OrchestrationApi {
            designs: kyron_foundation::containers::growable_vec::GrowableVec::default(),
            events: crate::events::events_provider::EventsProvider::default(),
            shutdown_events: GrowableVec::default(),
            _p: PhantomData,
        };
        api.designs.push(design);

        let design = crate::api::design::Design::new(design_tag, params);

        design.register_event("SomeUserEvent2".into()).unwrap();

        api.designs.push(design);

        api.design_done()
    }

    #[test]
    fn bind_events_as_global_works() {
        let mut api = setup_api_single_design();
        let mut deployment = Deployment::new(&mut api);
        let tag = Tag::from_str_static("SomeUserEvent");
        let result = deployment.bind_events_as_global("sys_event", &[tag]);
        assert!(result.is_ok());
    }

    #[test]
    fn bind_non_existing_events_as_global_cause_error() {
        let mut api = setup_api_multiple_design();
        let mut deployment = Deployment::new(&mut api);
        let tag = Tag::from_str_static("SomeUserEventNotExiting");
        let result = deployment.bind_events_as_global("sys_event", &[tag]);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommonErrors::NotFound);
    }

    #[test]
    fn bind_existing_events_as_global_in_single_deployment_works() {
        let mut api = setup_api_multiple_design();
        let mut deployment = Deployment::new(&mut api);
        let tag = Tag::from_str_static("SomeUserEvent2");
        let result = deployment.bind_events_as_global("sys_event", &[tag]);

        assert!(result.is_ok());
    }

    #[test]
    fn bind_events_as_local_works() {
        let mut api = setup_api_single_design();
        let mut deployment = Deployment::new(&mut api);
        let tag = Tag::from_str_static("SomeUserEvent");
        let result = deployment.bind_events_as_local(&[tag]);
        assert!(result.is_ok());
    }

    #[test]
    fn bind_non_existing_events_as_local_cause_error() {
        let mut api = setup_api_multiple_design();
        let mut deployment = Deployment::new(&mut api);
        let tag = Tag::from_str_static("SomeUserEventNotExiting");
        let result = deployment.bind_events_as_local(&[tag]);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CommonErrors::NotFound);
    }

    #[test]
    fn bind_existing_events_as_local_in_single_deployment_works() {
        let mut api = setup_api_multiple_design();
        let mut deployment = Deployment::new(&mut api);
        let tag = Tag::from_str_static("SomeUserEvent2");
        let result = deployment.bind_events_as_local(&[tag]);

        assert!(result.is_ok());
    }
}
