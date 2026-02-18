mod common;
use adguardhome_mcp_rs::tools::system;
use anyhow::Result;
use common::run_agnostic_test;

#[tokio::test]
async fn test_monitoring_tools() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            system::register(&mut reg);
        }

        let res = ctx
            .call_tool("manage_system", serde_json::json!({"action": "get_stats"}))
            .await?;
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("Queries:")
        );

        let res = ctx
            .call_tool(
                "manage_system",
                serde_json::json!({"action": "get_query_log", "limit": 5}),
            )
            .await?;
        assert!(!res["content"][0]["text"].as_str().unwrap().is_empty());

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_maintenance_tools_integration() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            system::register(&mut reg);
        }

        // Call clear_stats
        ctx.call_tool(
            "manage_system",
            serde_json::json!({"action": "clear_stats"}),
        )
        .await?;

        // Call clear_query_log
        ctx.call_tool(
            "manage_system",
            serde_json::json!({"action": "clear_query_log"}),
        )
        .await?;

        // Call create_backup
        let res = ctx
            .call_tool(
                "manage_system",
                serde_json::json!({"action": "create_backup"}),
            )
            .await?;
        let text = res["content"][0]["text"].as_str().unwrap();
        let backup_path_str = text.split("Backup: ").nth(1).unwrap().trim();
        let backup_path = std::path::Path::new(backup_path_str);
        assert!(backup_path.exists());

        // Call restore_backup
        ctx.call_tool(
            "manage_system",
            serde_json::json!({"action": "restore_backup", "file_path": backup_path_str}),
        )
        .await?;

        let _ = tokio::fs::remove_file(backup_path).await;

        // Call restart_service
        ctx.call_tool(
            "manage_system",
            serde_json::json!({"action": "restart_service"}),
        )
        .await?;

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_query_log_config_integration() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            system::register(&mut reg);
        }

        // Get current config
        let res = ctx.call_tool(
            "manage_system",
            serde_json::json!({"action": "get_query_log_config"}),
        )
        .await?;
        let ql_config: serde_json::Value =
            serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;

        // Toggle anonymize_client_ip
        let original_val = ql_config["anonymize_client_ip"].as_bool().unwrap();
        ctx.call_tool(
            "manage_system",
            serde_json::json!({"action": "set_query_log_config", "anonymize_client_ip": !original_val}),
        )
        .await?;

        let res = ctx.call_tool(
            "manage_system",
            serde_json::json!({"action": "get_query_log_config"}),
        )
        .await?;
        let updated_config: serde_json::Value =
            serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;
        assert_eq!(
            updated_config["anonymize_client_ip"].as_bool().unwrap(),
            !original_val
        );

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_system_info_integration() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            system::register(&mut reg);
        }

        // Get version info
        let res = ctx
            .call_tool(
                "manage_system",
                serde_json::json!({"action": "get_version_info"}),
            )
            .await?;
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("version")
        );

        // Trigger update
        let res = ctx
            .call_tool(
                "manage_system",
                serde_json::json!({"action": "update_adguard_home"}),
            )
            .await;

        match res {
            Ok(_) => println!("Update triggered successfully"),
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("no update available") {
                    println!("Verified: Update correctly rejected when none available.");
                } else {
                    return Err(e);
                }
            }
        }

        Ok(())
    })
    .await
}
