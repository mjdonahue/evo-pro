use crate::services::core::*;

#[derive(Clone)]
pub struct EventService {
    base: BaseService,
}

impl EventService {
    pub fn new(base: BaseService) -> Self {
        Self { base }
    }
}

impl EventServiceTrait for EventService {}
