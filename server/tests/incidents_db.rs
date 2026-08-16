mod common;

use sqlx::PgPool;
use vigil_server::features::incidents::{model::Severity, service};
use vigil_server::shared::error::AppError;

#[sqlx::test(migrations = "./migrations")]
async fn create_incident_succeeds_for_manager(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let (incident, _) =
        service::create_incident(&pool, team.id, user.id, "DB down", "", Severity::High, None)
            .await
            .unwrap();

    assert_eq!(incident.title, "DB down");
    assert_eq!(incident.team_id, team.id);
}

#[sqlx::test(migrations = "./migrations")]
async fn create_incident_fails_for_non_member(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = common::create_test_team(&pool, "Ops", user1.id).await;

    let result = service::create_incident(
        &pool,
        team.id,
        user2.id,
        "DB down",
        "",
        Severity::High,
        None,
    )
    .await;

    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn acknowledge_incident_transitions_state(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let (incident, _) =
        service::create_incident(&pool, team.id, user.id, "DB down", "", Severity::High, None)
            .await
            .unwrap();

    let updated = service::acknowledge_incident(&pool, team.id, incident.id, user.id)
        .await
        .unwrap();

    assert_eq!(
        updated.state,
        vigil_server::features::incidents::model::IncidentState::Acknowledged
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn acknowledge_incident_fails_if_already_acknowledged(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let (incident, _) =
        service::create_incident(&pool, team.id, user.id, "DB down", "", Severity::High, None)
            .await
            .unwrap();

    service::acknowledge_incident(&pool, team.id, incident.id, user.id)
        .await
        .unwrap();

    let result = service::acknowledge_incident(&pool, team.id, incident.id, user.id).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn escalate_incident_transitions_state_and_severity(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let (incident, _) =
        service::create_incident(&pool, team.id, user.id, "DB down", "", Severity::Low, None)
            .await
            .unwrap();

    service::acknowledge_incident(&pool, team.id, incident.id, user.id)
        .await
        .unwrap();

    let updated = service::escalate_incident(
        &pool,
        team.id,
        incident.id,
        user.id,
        Some(Severity::Critical),
    )
    .await
    .unwrap();

    assert_eq!(
        updated.state,
        vigil_server::features::incidents::model::IncidentState::Escalated
    );
    assert_eq!(updated.severity, Severity::Critical);
}

#[sqlx::test(migrations = "./migrations")]
async fn resolve_incident_sets_resolved_at(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let (incident, _) =
        service::create_incident(&pool, team.id, user.id, "DB down", "", Severity::High, None)
            .await
            .unwrap();

    let (updated, _) = service::resolve_incident(&pool, team.id, incident.id, user.id)
        .await
        .unwrap();

    assert!(updated.resolved_at.is_some());
}

#[sqlx::test(migrations = "./migrations")]
async fn resolve_incident_fails_if_already_resolved(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let (incident, _) =
        service::create_incident(&pool, team.id, user.id, "DB down", "", Severity::High, None)
            .await
            .unwrap();

    service::resolve_incident(&pool, team.id, incident.id, user.id)
        .await
        .unwrap();

    let result = service::resolve_incident(&pool, team.id, incident.id, user.id).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn assign_responder_succeeds(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = common::create_test_team(&pool, "Ops", user1.id).await;

    let code = vigil_server::features::teams::service::generate_invitation_code_for_team(
        &pool, team.id, user1.id,
    )
    .await
    .unwrap();
    vigil_server::features::teams::service::join_team(&pool, &code, user2.id)
        .await
        .unwrap();

    let (incident, _) = service::create_incident(
        &pool,
        team.id,
        user1.id,
        "DB down",
        "",
        Severity::High,
        None,
    )
    .await
    .unwrap();

    let updated = service::assign_responder(&pool, team.id, incident.id, user1.id, Some(user2.id))
        .await
        .unwrap();

    assert_eq!(updated.assigned_to, Some(user2.id));
}

#[sqlx::test(migrations = "./migrations")]
async fn assign_responder_fails_on_resolved_incident(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = common::create_test_team(&pool, "Ops", user1.id).await;

    let code = vigil_server::features::teams::service::generate_invitation_code_for_team(
        &pool, team.id, user1.id,
    )
    .await
    .unwrap();
    vigil_server::features::teams::service::join_team(&pool, &code, user2.id)
        .await
        .unwrap();

    let (incident, _) = service::create_incident(
        &pool,
        team.id,
        user1.id,
        "DB down",
        "",
        Severity::High,
        None,
    )
    .await
    .unwrap();
    service::resolve_incident(&pool, team.id, incident.id, user1.id)
        .await
        .unwrap();

    let result =
        service::assign_responder(&pool, team.id, incident.id, user1.id, Some(user2.id)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn add_timeline_entry_succeeds(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let (incident, _) =
        service::create_incident(&pool, team.id, user.id, "DB down", "", Severity::High, None)
            .await
            .unwrap();

    let entry = service::add_timeline_entry(&pool, team.id, incident.id, user.id, "Investigating")
        .await
        .unwrap();

    assert_eq!(entry.content, "Investigating");
    assert_eq!(entry.author_id, Some(user.id));
}

#[sqlx::test(migrations = "./migrations")]
async fn edit_timeline_entry_succeeds_for_author(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;
    let (team, _) = common::create_test_team(&pool, "Ops", user.id).await;

    let (incident, _) =
        service::create_incident(&pool, team.id, user.id, "DB down", "", Severity::High, None)
            .await
            .unwrap();

    let entry = service::add_timeline_entry(&pool, team.id, incident.id, user.id, "Initial note")
        .await
        .unwrap();

    let updated =
        service::edit_timeline_entry(&pool, team.id, incident.id, entry.id, user.id, "Updated")
            .await
            .unwrap();

    assert_eq!(updated.content, "Updated");
    assert!(updated.edited_at.is_some());
}

#[sqlx::test(migrations = "./migrations")]
async fn edit_timeline_entry_fails_for_non_author(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (team, _) = common::create_test_team(&pool, "Ops", user1.id).await;

    let code = vigil_server::features::teams::service::generate_invitation_code_for_team(
        &pool, team.id, user1.id,
    )
    .await
    .unwrap();
    vigil_server::features::teams::service::join_team(&pool, &code, user2.id)
        .await
        .unwrap();

    let (incident, _) = service::create_incident(
        &pool,
        team.id,
        user1.id,
        "DB down",
        "",
        Severity::High,
        None,
    )
    .await
    .unwrap();

    let entry = service::add_timeline_entry(&pool, team.id, incident.id, user1.id, "Initial note")
        .await
        .unwrap();

    let result =
        service::edit_timeline_entry(&pool, team.id, incident.id, entry.id, user2.id, "Hacked")
            .await;

    assert!(matches!(result, Err(AppError::Forbidden(_))));
}
