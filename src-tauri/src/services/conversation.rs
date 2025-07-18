use crate::services::core::*;

#[derive(Clone)]
pub struct ConversationService {
    base: BaseService,
}

impl ConversationService {
    pub fn new(base: BaseService) -> Self {
        Self { base }
    }
}

impl ConversationServiceTrait for ConversationService {}
