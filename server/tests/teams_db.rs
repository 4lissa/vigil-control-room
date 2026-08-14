mod common;

use sqlx::PgPool;
use vigil_server::features::teams::{model::Role, service};
use vigil_server::shared::error::AppError;

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
