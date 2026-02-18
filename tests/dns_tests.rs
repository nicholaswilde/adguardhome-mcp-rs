mod common;
use adguardhome_mcp_rs::tools::dns;
use anyhow::Result;
use common::run_agnostic_test;

#[tokio::test]
async fn test_dns_rewrites() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            dns::register(&mut reg);
        }

        let domain = "test.example.com".to_string();
        let answer = "1.1.1.1".to_string();

        ctx.call_tool(
            "manage_dns",
            serde_json::json!({"action": "add_rewrite", "domain": domain, "answer": answer}),
        )
        .await?;

        let res = ctx
            .call_tool("manage_dns", serde_json::json!({"action": "list_rewrites"}))
            .await?;
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains(&domain)
        );

        ctx.call_tool(
            "manage_dns",
            serde_json::json!({"action": "remove_rewrite", "domain": domain, "answer": answer}),
        )
        .await?;

        let res = ctx
            .call_tool("manage_dns", serde_json::json!({"action": "list_rewrites"}))
            .await?;
        assert!(
            !res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains(&domain)
        );

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_dns_config_tools() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            dns::register(&mut reg);
        }

        let res = ctx
            .call_tool("manage_dns", serde_json::json!({"action": "get_config"}))
            .await?;
        let _dns_info: serde_json::Value =
            serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;

        ctx.call_tool(
            "manage_dns",
            serde_json::json!({"action": "set_config", "upstream_dns": ["1.1.1.1"]}),
        )
        .await?;

        let res = ctx
            .call_tool("manage_dns", serde_json::json!({"action": "get_config"}))
            .await?;
        let updated_info: serde_json::Value =
            serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;
        assert_eq!(updated_info["upstream_dns"][0].as_str().unwrap(), "1.1.1.1");

        ctx.call_tool("manage_dns", serde_json::json!({"action": "clear_cache"}))
            .await?;

        Ok(())
    })
    .await
}
