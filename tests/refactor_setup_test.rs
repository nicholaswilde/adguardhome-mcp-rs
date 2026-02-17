mod common;
use common::TestContext;

#[tokio::test]
async fn test_context_setup() {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return;
    }

    let ctx = TestContext::new().await.expect("Failed to setup context");
    assert!(ctx.container.port() > 0);

    let status = ctx.client.get_status().await.expect("Failed to get status");
    assert!(!status.version.is_empty());
}
