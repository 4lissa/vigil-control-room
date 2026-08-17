mod common;

use sqlx::PgPool;
use vigil_server::features::teams::{model::Role, service};
use vigil_server::shared::error::AppError;

async fn create_team_with_member(pool: &PgPool) -> (uuid::Uuid, uuid::Uuid, uuid::Uuid) {
    let (user1, _) = common::create_test_user(pool).await;
    let (user2, _) = common::create_test_user_with(pool, "bob", "bob@example.com").await;
    let (team, _) = service::create_team(pool, "My Team", user1.id)
        .await
        .unwrap();

    let code = service::generate_invitation_code_for_team(pool, team.id, user1.id)
        .await
        .unwrap();
    service::join_team(pool, &code, user2.id).await.unwrap();

    (team.id, user1.id, user2.id)
}

#[sqlx::test(migrations = "./migrations")]
async fn create_team_succeeds(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, member) = service::create_team(&pool, "My Team", user.id)
        .await
        .unwrap();

    assert_eq!(team.name, "My Team");
    assert!(team.invitation_code.is_none());
    assert_eq!(member.user_id, user.id);
    assert_eq!(member.role, Role::Manager);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_my_teams_returns_only_user_teams(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;

    service::create_team(&pool, "Team A", user1.id)
        .await
        .unwrap();
    service::create_team(&pool, "Team B", user2.id)
        .await
        .unwrap();

    let teams = service::get_my_teams(&pool, user1.id).await.unwrap();
    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].name, "Team A");
}

#[sqlx::test(migrations = "./migrations")]
async fn generate_invitation_code_sets_code(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = service::create_team(&pool, "My Team", user.id)
        .await
        .unwrap();

    let code = service::generate_invitation_code_for_team(&pool, team.id, user.id)
        .await
        .unwrap();

    assert!(!code.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn generate_invitation_code_fails_for_non_manager(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = service::create_team(&pool, "My Team", user1.id)
        .await
        .unwrap();

    let result = service::generate_invitation_code_for_team(&pool, team.id, user2.id).await;
    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn join_team_succeeds_with_valid_code(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = service::create_team(&pool, "My Team", user1.id)
        .await
        .unwrap();

    let code = service::generate_invitation_code_for_team(&pool, team.id, user1.id)
        .await
        .unwrap();

    let (joined_team, member) = service::join_team(&pool, &code, user2.id).await.unwrap();

    assert_eq!(joined_team.id, team.id);
    assert_eq!(member.user_id, user2.id);
    assert_eq!(member.role, Role::Observer);
}

#[sqlx::test(migrations = "./migrations")]
async fn join_team_fails_with_invalid_code(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let result = service::join_team(&pool, "invalidcode", user.id).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn join_team_fails_when_already_member(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = service::create_team(&pool, "My Team", user.id)
        .await
        .unwrap();

    let code = service::generate_invitation_code_for_team(&pool, team.id, user.id)
        .await
        .unwrap();

    let result = service::join_team(&pool, &code, user.id).await;
    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn transfer_manager_succeeds(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = service::create_team(&pool, "My Team", user1.id)
        .await
        .unwrap();

    let code = service::generate_invitation_code_for_team(&pool, team.id, user1.id)
        .await
        .unwrap();
    service::join_team(&pool, &code, user2.id).await.unwrap();

    service::transfer_manager(&pool, team.id, user1.id, user2.id)
        .await
        .unwrap();

    let members = service::get_members(&pool, team.id, user2.id)
        .await
        .unwrap();
    let new_manager = members.iter().find(|m| m.user_id == user2.id).unwrap();
    let old_manager = members.iter().find(|m| m.user_id == user1.id).unwrap();

    assert_eq!(new_manager.role, Role::Manager);
    assert_eq!(old_manager.role, Role::Responder);
}

#[sqlx::test(migrations = "./migrations")]
async fn transfer_manager_fails_when_target_not_member(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = service::create_team(&pool, "My Team", user1.id)
        .await
        .unwrap();

    let result = service::transfer_manager(&pool, team.id, user1.id, user2.id).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn transfer_manager_fails_to_self(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = service::create_team(&pool, "My Team", user.id)
        .await
        .unwrap();

    let result = service::transfer_manager(&pool, team.id, user.id, user.id).await;
    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn kick_member_removes_the_member(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    service::kick_member(&pool, team_id, manager_id, member_id)
        .await
        .unwrap();

    let members = service::get_members(&pool, team_id, manager_id)
        .await
        .unwrap();
    assert!(!members.iter().any(|m| m.user_id == member_id));
}

#[sqlx::test(migrations = "./migrations")]
async fn kick_member_fails_for_non_manager(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    let result = service::kick_member(&pool, team_id, member_id, manager_id).await;
    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn kick_member_fails_to_self(pool: PgPool) {
    let (team_id, manager_id, _) = create_team_with_member(&pool).await;

    let result = service::kick_member(&pool, team_id, manager_id, manager_id).await;
    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn kick_member_fails_for_non_member_target(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (other, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = service::create_team(&pool, "My Team", user.id)
        .await
        .unwrap();

    let result = service::kick_member(&pool, team.id, user.id, other.id).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn kicked_member_can_rejoin_immediately(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    service::kick_member(&pool, team_id, manager_id, member_id)
        .await
        .unwrap();

    let code = service::generate_invitation_code_for_team(&pool, team_id, manager_id)
        .await
        .unwrap();

    let result = service::join_team(&pool, &code, member_id).await;
    assert!(result.is_ok());
}

#[sqlx::test(migrations = "./migrations")]
async fn ban_member_removes_the_member(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    service::ban_member(&pool, team_id, manager_id, member_id, None)
        .await
        .unwrap();

    let members = service::get_members(&pool, team_id, manager_id)
        .await
        .unwrap();
    assert!(!members.iter().any(|m| m.user_id == member_id));
}

#[sqlx::test(migrations = "./migrations")]
async fn ban_member_fails_for_non_manager(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    let result = service::ban_member(&pool, team_id, member_id, manager_id, None).await;
    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn ban_member_fails_to_self(pool: PgPool) {
    let (team_id, manager_id, _) = create_team_with_member(&pool).await;

    let result = service::ban_member(&pool, team_id, manager_id, manager_id, None).await;
    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn ban_member_fails_for_expiration_in_the_past(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    let result = service::ban_member(&pool, team_id, manager_id, member_id, Some(1)).await;
    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn ban_member_fails_for_unknown_user(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = service::create_team(&pool, "My Team", user.id)
        .await
        .unwrap();

    let result = service::ban_member(&pool, team.id, user.id, uuid::Uuid::now_v7(), None).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn banned_member_cannot_rejoin(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    service::ban_member(&pool, team_id, manager_id, member_id, None)
        .await
        .unwrap();

    let code = service::generate_invitation_code_for_team(&pool, team_id, manager_id)
        .await
        .unwrap();

    let result = service::join_team(&pool, &code, member_id).await;
    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn unban_member_lifts_the_ban(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    service::ban_member(&pool, team_id, manager_id, member_id, None)
        .await
        .unwrap();
    service::unban_member(&pool, team_id, manager_id, member_id)
        .await
        .unwrap();

    let code = service::generate_invitation_code_for_team(&pool, team_id, manager_id)
        .await
        .unwrap();

    let result = service::join_team(&pool, &code, member_id).await;
    assert!(result.is_ok());
}

#[sqlx::test(migrations = "./migrations")]
async fn unban_member_fails_for_non_manager(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    let result = service::unban_member(&pool, team_id, member_id, manager_id).await;
    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_active_bans_returns_banned_users(pool: PgPool) {
    let (team_id, manager_id, member_id) = create_team_with_member(&pool).await;

    service::ban_member(&pool, team_id, manager_id, member_id, None)
        .await
        .unwrap();

    let bans = service::get_active_bans(&pool, team_id, manager_id)
        .await
        .unwrap();

    assert_eq!(bans.len(), 1);
    assert_eq!(bans[0].user_id, member_id);
    assert_eq!(bans[0].username, "bob");
}

#[sqlx::test(migrations = "./migrations")]
async fn get_active_bans_fails_for_non_manager(pool: PgPool) {
    let (team_id, _, member_id) = create_team_with_member(&pool).await;

    let result = service::get_active_bans(&pool, team_id, member_id).await;
    assert!(matches!(result, Err(AppError::Forbidden(_))));
}
