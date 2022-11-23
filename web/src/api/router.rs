use crate::api::service;
use actix_web::{post, web, Responder};
use rusty_gate::types::CheckRolesOfMembersRequest;

#[post("/checkRolesOfMembers")]
async fn check_roles_of_members(body: web::Json<CheckRolesOfMembersRequest>) -> impl Responder {
    log::info!("check_roles_of_members - {:?}", body);
    web::Json(
        service::check_roles_of_members(
            &body.users,
            &body.roles,
            body.send_details.unwrap_or_default(),
        )
        .await,
    )
}
