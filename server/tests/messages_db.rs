mod common;

use sqlx::PgPool;
use vigil_server::features::messages::service;
use vigil_server::shared::error::AppError;

#[sqlx::test(migrations = "./migrations")]
async fn send_message_succeeds_between_teammates(pool: PgPool) {
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

    let message = service::send_message(&pool, user1.id, user2.id, "Hey, got a minute?")
        .await
        .unwrap();

    assert_eq!(message.content, "Hey, got a minute?");
    assert_eq!(message.sender_id, user1.id);
    assert_eq!(message.recipient_id, user2.id);
}

#[sqlx::test(migrations = "./migrations")]
async fn send_message_fails_without_shared_team(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;

    let result = service::send_message(&pool, user1.id, user2.id, "Hey").await;

    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn send_message_fails_to_self(pool: PgPool) {
    let (user, _) = common::create_test_user(&pool).await;

    let result = service::send_message(&pool, user.id, user.id, "Hey").await;

    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_conversation_returns_messages_in_chronological_order(pool: PgPool) {
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

    service::send_message(&pool, user1.id, user2.id, "first")
        .await
        .unwrap();
    service::send_message(&pool, user2.id, user1.id, "second")
        .await
        .unwrap();

    let conversation = service::get_conversation(&pool, user1.id, user2.id)
        .await
        .unwrap();

    assert_eq!(conversation.len(), 2);
    assert_eq!(conversation[0].content, "first");
    assert_eq!(conversation[1].content, "second");
}

#[sqlx::test(migrations = "./migrations")]
async fn get_conversation_is_symmetric_for_both_participants(pool: PgPool) {
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

    service::send_message(&pool, user1.id, user2.id, "hi")
        .await
        .unwrap();

    let as_sender = service::get_conversation(&pool, user1.id, user2.id)
        .await
        .unwrap();
    let as_recipient = service::get_conversation(&pool, user2.id, user1.id)
        .await
        .unwrap();

    assert_eq!(as_sender.len(), 1);
    assert_eq!(as_recipient.len(), 1);
    assert_eq!(as_sender[0].id, as_recipient[0].id);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_conversation_fails_without_shared_team(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;

    let result = service::get_conversation(&pool, user1.id, user2.id).await;

    assert!(matches!(result, Err(AppError::Forbidden(_))));
}

#[sqlx::test(migrations = "./migrations")]
async fn list_conversations_returns_latest_message_per_counterpart(pool: PgPool) {
    let (user1, _) = common::create_test_user(&pool).await;
    let (user2, _) = common::create_test_user_with(&pool, "bob", "bob@example.com").await;
    let (user3, _) = common::create_test_user_with(&pool, "carol", "carol@example.com").await;
    let (team, _) = common::create_test_team(&pool, "Ops", user1.id).await;

    for member_id in [user2.id, user3.id] {
        let code = vigil_server::features::teams::service::generate_invitation_code_for_team(
            &pool, team.id, user1.id,
        )
        .await
        .unwrap();
        vigil_server::features::teams::service::join_team(&pool, &code, member_id)
            .await
            .unwrap();
    }

    service::send_message(&pool, user1.id, user2.id, "first with bob")
        .await
        .unwrap();
    service::send_message(&pool, user1.id, user3.id, "hi carol")
        .await
        .unwrap();
    service::send_message(&pool, user2.id, user1.id, "second with bob")
        .await
        .unwrap();

    let conversations = service::list_conversations(&pool, user1.id).await.unwrap();

    assert_eq!(conversations.len(), 2);
    assert_eq!(conversations[0].content, "second with bob");
    assert_eq!(conversations[1].content, "hi carol");
}

#[sqlx::test(migrations = "./migrations")]
async fn list_conversations_hides_counterparts_no_longer_sharing_a_team(pool: PgPool) {
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

    service::send_message(&pool, user1.id, user2.id, "hi")
        .await
        .unwrap();

    sqlx::query!(
        "DELETE FROM team_members WHERE team_id = $1 AND user_id = $2",
        team.id,
        user2.id,
    )
    .execute(&pool)
    .await
    .unwrap();

    let conversations = service::list_conversations(&pool, user1.id).await.unwrap();

    assert!(conversations.is_empty());
}
