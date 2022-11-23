use rusty_gate::{
    requirements::check_access,
    types::{CheckRolesOfMembersResult, Role, User},
};

pub async fn check_roles_of_members(
    users: &[User],
    roles: &[Role],
    send_details: bool,
) -> Vec<CheckRolesOfMembersResult> {
    futures::future::join_all(roles.iter().map(|role| async {
        let result = check_access(users, &role.requirements, &role.logic, send_details).await;

        CheckRolesOfMembersResult {
            role_id: role.id.expect("Unwrapping the ID should be fine"),
            users: result.accesses,
            errors: result.errors,
        }
    }))
    .await
}
