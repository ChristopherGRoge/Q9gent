use super::*;

#[tokio::test]
async fn test_session_store_create() {
    let temp_dir = tempfile::tempdir().unwrap();
    let store = SessionStore::new(temp_dir.path());

    let metadata = store
        .create_session("test_agent".to_string())
        .await
        .unwrap();

    assert_eq!(metadata.agent_type, "test_agent");
    assert!(!metadata.session_id.is_empty());
    assert!(metadata.created_at > 0);
    assert_eq!(metadata.created_at, metadata.last_used);
}

#[tokio::test]
async fn test_session_store_save_load() {
    let temp_dir = tempfile::tempdir().unwrap();
    let store = SessionStore::new(temp_dir.path());

    let metadata = SessionMetadata {
        session_id: "test-123".to_string(),
        agent_type: "test".to_string(),
        created_at: 1000,
        last_used: 2000,
    };

    store.save_session(&metadata).await.unwrap();
    let loaded = store.load_session("test-123").await.unwrap();

    assert_eq!(loaded.session_id, metadata.session_id);
    assert_eq!(loaded.agent_type, metadata.agent_type);
    assert_eq!(loaded.created_at, metadata.created_at);
    assert_eq!(loaded.last_used, metadata.last_used);
}

#[tokio::test]
async fn test_session_store_touch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let store = SessionStore::new(temp_dir.path());

    let metadata = store.create_session("test".to_string()).await.unwrap();
    let original_last_used = metadata.last_used;

    // Wait to ensure timestamp changes (timestamps are in seconds)
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    store.touch_session(&metadata.session_id).await.unwrap();
    let updated = store.load_session(&metadata.session_id).await.unwrap();

    assert!(updated.last_used > original_last_used);
}

#[tokio::test]
async fn test_session_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let store = SessionStore::new(temp_dir.path());

    let result = store.load_session("nonexistent").await;
    assert!(matches!(result, Err(AppError::SessionNotFound(_))));
}
