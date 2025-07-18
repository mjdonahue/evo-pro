use crate::services::core::*;

#[derive(Clone)]
pub struct PlanService {
    base: BaseService,
}

impl PlanService {
    pub fn new(base: BaseService) -> Self {
        Self { base }
    }
}

impl PlanServiceTrait for PlanService {}
