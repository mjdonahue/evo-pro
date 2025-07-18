use crate::services::core::*;

#[derive(Clone)]
pub struct AgentService {
    base: BaseService,
}

impl AgentService {
    pub fn new(base: BaseService) -> Self {
        Self { base }
    }
}

impl AgentServiceTrait for AgentService {}
