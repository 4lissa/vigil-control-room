mod common;

use sqlx::PgPool;
use vigil_server::features::incidents::{model::Severity, service as incidents_service};
use vigil_server::features::releases::{model::ReleaseState, service};
use vigil_server::shared::error::AppError;

#[sqlx::test(migrations = "./migrations")]
async fn create_release_succeeds_for_manager(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release = service::create_release(
        &pool,
        team.id,
        user.id,
        "v1.0.0",
        &["build".to_string(), "staging".to_string()],
    )
    .await
    .unwrap();

    assert_eq!(release.name, "v1.0.0");
    assert_eq!(release.state, ReleaseState::Created);

    let steps = service::get_release_steps(&pool, team.id, release.id, user.id)
        .await
        .unwrap();
    assert_eq!(steps.len(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_release_fails_for_non_manager(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = common::create_test_team(&pool, "Ops", user1.id).await;

    let result =
        service::create_release(&pool, team.id, user2.id, "v1.0.0", &["build".to_string()]).await;

    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn validate_first_step_moves_release_to_in_progress(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release = service::create_release(
        &pool,
        team.id,
        user.id,
        "v1.0.0",
        &["build".to_string(), "staging".to_string()],
    )
    .await
    .unwrap();
    let steps = service::get_release_steps(&pool, team.id, release.id, user.id)
        .await
        .unwrap();

    let (_, new_state) = service::validate_step(&pool, team.id, release.id, steps[0].id, user.id)
        .await
        .unwrap();

    assert_eq!(new_state, Some(ReleaseState::InProgress));
}

#[sqlx::test(migrations = "./migrations")]
async fn validate_step_out_of_order_fails(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release = service::create_release(
        &pool,
        team.id,
        user.id,
        "v1.0.0",
        &["build".to_string(), "staging".to_string()],
    )
    .await
    .unwrap();
    let steps = service::get_release_steps(&pool, team.id, release.id, user.id)
        .await
        .unwrap();

    let result = service::validate_step(&pool, team.id, release.id, steps[1].id, user.id).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn validate_last_step_completes_release(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release =
        service::create_release(&pool, team.id, user.id, "v1.0.0", &["build".to_string()])
            .await
            .unwrap();
    let steps = service::get_release_steps(&pool, team.id, release.id, user.id)
        .await
        .unwrap();

    let (_, new_state) = service::validate_step(&pool, team.id, release.id, steps[0].id, user.id)
        .await
        .unwrap();

    assert_eq!(new_state, Some(ReleaseState::Completed));
}

#[sqlx::test(migrations = "./migrations")]
async fn cancel_release_succeeds_for_manager(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release =
        service::create_release(&pool, team.id, user.id, "v1.0.0", &["build".to_string()])
            .await
            .unwrap();

    let cancelled = service::cancel_release(&pool, team.id, release.id, user.id)
        .await
        .unwrap();

    assert_eq!(cancelled.state, ReleaseState::Cancelled);
}

#[sqlx::test(migrations = "./migrations")]
async fn cancel_release_fails_once_completed(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release =
        service::create_release(&pool, team.id, user.id, "v1.0.0", &["build".to_string()])
            .await
            .unwrap();
    let steps = service::get_release_steps(&pool, team.id, release.id, user.id)
        .await
        .unwrap();
    service::validate_step(&pool, team.id, release.id, steps[0].id, user.id)
        .await
        .unwrap();

    let result = service::cancel_release(&pool, team.id, release.id, user.id).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn create_incident_blocks_in_progress_release(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release = service::create_release(
        &pool,
        team.id,
        user.id,
        "v1.0.0",
        &["build".to_string(), "staging".to_string()],
    )
    .await
    .unwrap();
    let steps = service::get_release_steps(&pool, team.id, release.id, user.id)
        .await
        .unwrap();
    service::validate_step(&pool, team.id, release.id, steps[0].id, user.id)
        .await
        .unwrap();

    let (_, blocked_state) = incidents_service::create_incident(
        &pool,
        team.id,
        user.id,
        "DB down",
        "",
        Severity::High,
        Some(release.id),
    )
    .await
    .unwrap();

    assert_eq!(blocked_state, Some(ReleaseState::Blocked));

    let release = service::get_release(&pool, team.id, release.id, user.id)
        .await
        .unwrap();
    assert_eq!(release.state, ReleaseState::Blocked);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_incident_does_not_block_release_that_is_not_in_progress(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release =
        service::create_release(&pool, team.id, user.id, "v1.0.0", &["build".to_string()])
            .await
            .unwrap();

    let (_, blocked_state) = incidents_service::create_incident(
        &pool,
        team.id,
        user.id,
        "DB down",
        "",
        Severity::High,
        Some(release.id),
    )
    .await
    .unwrap();

    assert_eq!(blocked_state, None);
}

#[sqlx::test(migrations = "./migrations")]
async fn validate_step_fails_with_unresolved_incident_even_if_release_not_blocked(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release =
        service::create_release(&pool, team.id, user.id, "v1.0.0", &["build".to_string()])
            .await
            .unwrap();

    incidents_service::create_incident(
        &pool,
        team.id,
        user.id,
        "DB down",
        "",
        Severity::High,
        Some(release.id),
    )
    .await
    .unwrap();

    let steps = service::get_release_steps(&pool, team.id, release.id, user.id)
        .await
        .unwrap();

    let result = service::validate_step(&pool, team.id, release.id, steps[0].id, user.id).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn resolving_last_active_incident_unblocks_release(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release = service::create_release(
        &pool,
        team.id,
        user.id,
        "v1.0.0",
        &["build".to_string(), "staging".to_string()],
    )
    .await
    .unwrap();
    let steps = service::get_release_steps(&pool, team.id, release.id, user.id)
        .await
        .unwrap();
    service::validate_step(&pool, team.id, release.id, steps[0].id, user.id)
        .await
        .unwrap();

    let (incident, _) = incidents_service::create_incident(
        &pool,
        team.id,
        user.id,
        "DB down",
        "",
        Severity::High,
        Some(release.id),
    )
    .await
    .unwrap();

    let (_, unblocked_state) =
        incidents_service::resolve_incident(&pool, team.id, incident.id, user.id)
            .await
            .unwrap();

    assert_eq!(unblocked_state, Some(ReleaseState::InProgress));
}

#[sqlx::test(migrations = "./migrations")]
async fn resolving_one_of_two_active_incidents_keeps_release_blocked(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let release = service::create_release(
        &pool,
        team.id,
        user.id,
        "v1.0.0",
        &["build".to_string(), "staging".to_string()],
    )
    .await
    .unwrap();
    let steps = service::get_release_steps(&pool, team.id, release.id, user.id)
        .await
        .unwrap();
    service::validate_step(&pool, team.id, release.id, steps[0].id, user.id)
        .await
        .unwrap();

    let (incident1, _) = incidents_service::create_incident(
        &pool,
        team.id,
        user.id,
        "DB down",
        "",
        Severity::High,
        Some(release.id),
    )
    .await
    .unwrap();
    incidents_service::create_incident(
        &pool,
        team.id,
        user.id,
        "API down",
        "",
        Severity::High,
        Some(release.id),
    )
    .await
    .unwrap();

    let (_, unblocked_state) =
        incidents_service::resolve_incident(&pool, team.id, incident1.id, user.id)
            .await
            .unwrap();

    assert_eq!(unblocked_state, None);

    let release = service::get_release(&pool, team.id, release.id, user.id)
        .await
        .unwrap();
    assert_eq!(release.state, ReleaseState::Blocked);
}
