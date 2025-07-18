use crate::services::core::*;

#[derive(Clone)]
pub struct MessageService {
    base: BaseService,
}

impl MessageService {
    pub fn new(base: BaseService) -> Self {
        Self { base }
    }
}

impl MessageServiceTrait for MessageService {}
