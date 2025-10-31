use std::sync::Arc;

pub mod enforcer;
pub mod watch;

#[derive(Clone)]
pub struct Wall {
    pub enforcer: Arc<enforcer::Enforcer>,
    pub watch: Arc<watch::Watch>,
}
