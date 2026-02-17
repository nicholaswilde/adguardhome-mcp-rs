mod common;
use adguardhome_mcp_rs::tools::protection;
use anyhow::Result;
use common::run_agnostic_test;

#[tokio::test]
async fn test_protection_tools() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            protection::register(&mut reg);
        }

        // Toggle global protection
        ctx.call_tool(
            "manage_protection",
            serde_json::json!({"action": "toggle_feature", "feature": "global", "enabled": false}),
        )
        .await?;

        let mut success = false;
        for _ in 0..10 {
            if !ctx.client.get_status().await?.protection_enabled {
                success = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        assert!(success, "Global protection did not disable in time");

        ctx.call_tool(
            "manage_protection",
            serde_json::json!({"action": "toggle_feature", "feature": "global", "enabled": true}),
        )
        .await?;

        let mut success = false;
        for _ in 0..10 {
            if ctx.client.get_status().await?.protection_enabled {
                success = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        assert!(success, "Global protection did not enable in time");

        // Other toggles
        ctx.call_tool(
            "manage_protection",
            serde_json::json!({"action": "toggle_feature", "feature": "safe_search", "enabled": true}),
        )
        .await?;
        ctx.call_tool(
            "manage_protection",
            serde_json::json!({"action": "toggle_feature", "feature": "safe_browsing", "enabled": true}),
        )
        .await?;
        ctx.call_tool(
            "manage_protection",
            serde_json::json!({"action": "toggle_feature", "feature": "parental_control", "enabled": true}),
        )
        .await?;

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_advanced_protection_integration() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            protection::register(&mut reg);
        }

        // Test Safe Search Settings
        let res = ctx
            .call_tool(
                "manage_protection",
                serde_json::json!({"action": "get_config"}),
            )
            .await?;
        let text = res["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Expected string content, got {:?}", res))?;
        let config_val: serde_json::Value = serde_json::from_str(text)?;
        let mut safe_search = config_val["safe_search"].clone();

        // Toggle one of them
        let original_bing = safe_search["bing"].as_bool().unwrap();
        safe_search["bing"] = serde_json::json!(!original_bing);
        ctx.call_tool(
            "manage_protection",
            serde_json::json!({"action": "set_config", "safe_search": safe_search}),
        )
        .await?;

        let res = ctx
            .call_tool(
                "manage_protection",
                serde_json::json!({"action": "get_config"}),
            )
            .await?;
        let text = res["content"][0]["text"].as_str().unwrap();
        let updated_config: serde_json::Value = serde_json::from_str(text)?;
        assert_eq!(
            updated_config["safe_search"]["bing"].as_bool().unwrap(),
            !original_bing
        );

        // Test Parental Settings
        let mut parental = config_val["parental_control"].clone();
        let original_parental = parental["enabled"].as_bool().unwrap();
        parental["enabled"] = serde_json::json!(!original_parental);
        ctx.call_tool(
            "manage_protection",
            serde_json::json!({"action": "set_config", "parental_control": parental}),
        )
        .await?;

        let res = ctx
            .call_tool(
                "manage_protection",
                serde_json::json!({"action": "get_config"}),
            )
            .await?;
        let text = res["content"][0]["text"].as_str().unwrap();
        let updated_config: serde_json::Value = serde_json::from_str(text)?;
        assert_eq!(
            updated_config["parental_control"]["enabled"]
                .as_bool()
                .unwrap(),
            !original_parental
        );

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_tls_config_integration() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            protection::register(&mut reg);
        }

        // Call get_tls_config
        let res = ctx
            .call_tool(
                "manage_protection",
                serde_json::json!({"action": "get_tls_config"}),
            )
            .await?;
        let text = res["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Expected string content, got {:?}", res))?;
        let status: serde_json::Value = serde_json::from_str(text)?;
        assert!(!status["enabled"].as_bool().unwrap()); // Default is usually disabled

        Ok(())
    })
    .await
}
