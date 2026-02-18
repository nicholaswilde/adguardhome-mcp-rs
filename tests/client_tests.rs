mod common;
use adguardhome_mcp_rs::tools::clients;
use anyhow::Result;
use common::run_agnostic_test;

#[tokio::test]
async fn test_client_tools() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            clients::register(&mut reg);
        }

        let res = ctx
            .call_tool(
                "manage_clients",
                serde_json::json!({"action": "list_clients"}),
            )
            .await?;
        assert!(res["content"][0]["text"].as_str().unwrap().contains("["));

        let test_client_name = "Integration Test Client".to_string();
        ctx.call_tool(
            "manage_clients",
            serde_json::json!({
                "action": "add_client",
                "name": test_client_name,
                "ids": ["1.2.3.4"]
            }),
        )
        .await?;

        let res = ctx
            .call_tool(
                "manage_clients",
                serde_json::json!({"action": "get_client_info", "identifier": test_client_name}),
            )
            .await?;
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains(&test_client_name)
        );

        ctx.call_tool(
            "manage_clients",
            serde_json::json!({"action": "delete_client", "name": test_client_name}),
        )
        .await?;

        let res = ctx
            .call_tool(
                "manage_clients",
                serde_json::json!({"action": "list_clients"}),
            )
            .await?;
        assert!(
            !res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains(&test_client_name)
        );

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_access_control_tools() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            clients::register(&mut reg);
        }

        let res = ctx
            .call_tool(
                "manage_clients",
                serde_json::json!({"action": "get_access_list"}),
            )
            .await?;
        let mut list: serde_json::Value =
            serde_json::from_str(res["content"][0]["text"].as_str().unwrap())?;

        let blocked_hosts = list["blocked_hosts"].as_array_mut().unwrap();
        let domain = serde_json::json!("test-blocked.com");
        if !blocked_hosts.contains(&domain) {
            blocked_hosts.push(domain);
        }

        ctx.call_tool(
            "manage_clients",
            serde_json::json!({"action": "update_access_list", "blocked_hosts": blocked_hosts}),
        )
        .await?;

        let res = ctx
            .call_tool(
                "manage_clients",
                serde_json::json!({"action": "get_access_list"}),
            )
            .await?;
        assert!(
            res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("test-blocked.com")
        );

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_dhcp_tools() -> Result<()> {
    if std::env::var("RUN_DOCKER_TESTS").unwrap_or_default() != "true" {
        return Ok(());
    }

    run_agnostic_test(|ctx| async move {
        {
            let mut reg = ctx.registry.lock().unwrap();
            clients::register(&mut reg);
        }

        // DHCP might not be enabled in the default container, but we can check if the endpoint responds
        ctx.call_tool(
            "manage_clients",
            serde_json::json!({"action": "list_dhcp_leases"}),
        )
        .await?;

        Ok(())
    })
    .await
}
