use adguardhome_mcp_rs::run;

#[tokio::test]
async fn test_run_help() {
    // Calling run with --help usually exits or returns an error because of clap
    // but we can try to run it with some basic args.
    let args = vec!["adguardhome-mcp-rs".to_string(), "--help".to_string()];
    let result = run(args).await;
    // Clap --help usually exits, so this might not return if run normally.
    // But since we are calling it as a function, it might return an error.
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_http_mode() {
    let args = vec![
        "adguardhome-mcp-rs".to_string(),
        "--adguard-host".to_string(),
        "localhost".to_string(),
        "--transport".to_string(),
        "http".to_string(),
        "--http-port".to_string(),
        "0".to_string(),
    ];
    let handle = tokio::spawn(async move {
        let _ = run(args).await;
    });
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    handle.abort();
}
