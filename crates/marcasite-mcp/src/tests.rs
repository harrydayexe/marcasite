//! In-process protocol tests: an rmcp client talks to the server over an in-memory duplex.

use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, ClientConfig, GetPromptRequestParams, ReadResourceRequestParams,
    ResourceContents,
};
use rmcp::service::RunningService;
use rmcp::{ClientHandler, RoleClient, ServiceExt as _};
use serde_json::{Value, json};

use crate::docs::Docs;
use crate::index::Index;
use crate::server::DocsServer;

#[derive(Debug, Clone, Default)]
struct Client;

impl ClientHandler for Client {
    fn get_info(&self) -> ClientConfig {
        ClientConfig::default()
    }
}

async fn connect() -> (
    RunningService<RoleClient, Client>,
    tokio::task::JoinHandle<()>,
) {
    let (server_io, client_io) = tokio::io::duplex(1 << 16);
    let server = DocsServer::new(Arc::new(Docs::new(Index::embedded().unwrap())));
    let task = tokio::spawn(async move {
        server
            .serve(server_io)
            .await
            .unwrap()
            .waiting()
            .await
            .unwrap();
    });
    (Client.serve(client_io).await.unwrap(), task)
}

/// Calls a tool and returns (text, is_error).
async fn call(
    client: &RunningService<RoleClient, Client>,
    tool: &str,
    args: Value,
) -> (String, bool) {
    let Value::Object(args) = args else {
        panic!("arguments must be an object")
    };
    let result = client
        .call_tool(CallToolRequestParams::new(tool.to_owned()).with_arguments(args))
        .await
        .unwrap();
    let text = result
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.clone())
        .unwrap();
    (text, result.is_error == Some(true))
}

fn resource_text(contents: &[ResourceContents]) -> &str {
    match contents.first().unwrap() {
        ResourceContents::TextResourceContents { text, .. } => text,
        other => panic!("expected text, got {other:?}"),
    }
}

#[tokio::test]
async fn tools_resources_and_prompts_over_the_protocol() {
    let (client, task) = connect().await;

    let info = client.peer_info().unwrap();
    assert_eq!(info.server_info.as_ref().unwrap().name, "marcasite-mcp");
    assert!(
        info.instructions
            .as_deref()
            .unwrap()
            .contains("find_endpoint")
    );

    let tools = client.list_all_tools().await.unwrap();
    let mut names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    names.sort();
    assert_eq!(
        names,
        ["find_endpoint", "get_item", "list_module", "search"]
    );
    let search = tools.iter().find(|t| t.name == "search").unwrap();
    let props = search.input_schema.get("properties").unwrap();
    for p in ["query", "module", "kind", "limit"] {
        assert!(props.get(p).is_some(), "search has no `{p}` parameter");
    }

    let (text, err) = call(
        &client,
        "search",
        json!({"query": "GammaClient", "limit": 3}),
    )
    .await;
    assert!(!err);
    assert!(
        text.contains("- `marcasite::gamma::GammaClient` (struct)"),
        "{text}"
    );

    let (text, err) = call(&client, "get_item", json!({"path": "GammaClient::get_tag"})).await;
    assert!(!err);
    assert!(
        text.contains("pub fn get_tag(&self, id: impl Into<TagId>) -> GetTag"),
        "{text}"
    );

    let (text, err) = call(&client, "get_item", json!({"path": "NoSuchThing"})).await;
    assert!(err, "{text}");

    let (text, err) = call(&client, "list_module", json!({"module": "gamma"})).await;
    assert!(!err);
    assert!(text.starts_with("# Module `marcasite::gamma`"), "{text}");
    assert!(text.contains("- `GammaClient`:"), "{text}");

    let (_, err) = call(&client, "list_module", json!({"module": "nope"})).await;
    assert!(err);

    let (text, err) = call(
        &client,
        "find_endpoint",
        json!({"query": "/tags/{id}", "method": "GET", "service": "gamma"}),
    )
    .await;
    assert!(!err);
    assert!(
        text.contains("marcasite::gamma::GammaClient::get_tag"),
        "{text}"
    );

    // A call missing a required parameter (`query`) fails instead of guessing.
    let result = client
        .call_tool(
            CallToolRequestParams::new("search")
                .with_arguments(json!({"limit": 3}).as_object().unwrap().clone()),
        )
        .await;
    assert!(result.is_err() || result.unwrap().is_error == Some(true));

    let resources = client.list_all_resources().await.unwrap();
    let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
    assert!(uris.contains(&"marcasite://sdk/marcasite"));
    assert!(uris.contains(&"marcasite://sdk/gamma"));
    assert!(uris.contains(&"marcasite://sdk/endpoints"));
    for uri in uris {
        let read = client
            .read_resource(ReadResourceRequestParams::new(uri))
            .await
            .unwrap();
        assert!(!resource_text(&read.contents).is_empty(), "{uri}");
    }
    let templates = client.list_all_resource_templates().await.unwrap();
    assert_eq!(templates[0].uri_template, "marcasite://sdk/{module}");
    assert!(
        client
            .read_resource(ReadResourceRequestParams::new("marcasite://sdk/nope"))
            .await
            .is_err()
    );

    let prompts = client.list_all_prompts().await.unwrap();
    assert_eq!(prompts[0].name, "use-marcasite");
    let prompt = client
        .get_prompt(GetPromptRequestParams::new("use-marcasite"))
        .await
        .unwrap();
    assert_eq!(prompt.messages.len(), 1);

    client.cancel().await.unwrap();
    task.await.unwrap();
}
