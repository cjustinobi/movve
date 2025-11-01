use std::sync::Arc;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}
