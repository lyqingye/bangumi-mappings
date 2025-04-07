use std::sync::Arc;

use actix_web::web;

use crate::api::{get_mapping, query_mappings, update_verification_status};
use crate::server::AppState;
pub fn configure_app(cfg: &mut web::ServiceConfig, state: Arc<AppState>) {
    cfg.app_data(web::Data::new(state.clone()))
        .service(query_mappings)
        .service(update_verification_status)
        .service(get_mapping);
}
