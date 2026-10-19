//! The MCP server: tools, resources and the `use-marcasite` prompt over [`Docs`].
//!
//! Tool names, parameter names and resource URIs are a compatibility surface: renaming or
//! removing one is a breaking change.

use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    GetPromptRequestParams, GetPromptResponse, GetPromptResult, Implementation, ListPromptsResult,
    ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParams, Prompt,
    PromptMessage, ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult, Resource,
    ResourceContents, ResourceTemplate, Role, ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::docs::Docs;

/// URI prefix of the module resources.
const MODULE_URI: &str = "marcasite://sdk/";
/// URI of the endpoint coverage resource.
const ENDPOINTS_URI: &str = "marcasite://sdk/endpoints";
const PROMPT: &str = "use-marcasite";
const MARKDOWN: &str = "text/markdown";
/// Default and maximum number of search results.
const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 100;

/// Parameters of the `search` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct SearchParams {
    /// Words to look for in item names, paths and docs, e.g. `order book`, `list_markets`,
    /// `Decimal`.
    pub(crate) query: String,
    /// Only search this module, e.g. `gamma`, `clob`, `data`, `ws`, `types`.
    #[serde(default)]
    pub(crate) module: Option<String>,
    /// Only return items of this kind: `struct`, `enum`, `trait`, `type`, `constant`, `fn`,
    /// `method`, `associated_const` or `module`.
    #[serde(default)]
    pub(crate) kind: Option<String>,
    /// Maximum number of results (default 20, at most 100).
    #[serde(default)]
    pub(crate) limit: Option<u32>,
}

/// Parameters of the `get_item` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct GetItemParams {
    /// Full or short path of an item or module, e.g. `marcasite::gamma::GammaClient`,
    /// `GammaClient::list_markets`, `Event`, `gamma`.
    pub(crate) path: String,
}

/// Parameters of the `list_module` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ListModuleParams {
    /// Module name or path, e.g. `gamma`, `marcasite::clob`, or `marcasite` for the crate
    /// root.
    pub(crate) module: String,
}

/// Parameters of the `find_endpoint` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct FindEndpointParams {
    /// A Polymarket doc URL (`https://docs.polymarket.com/api-reference/tags/get-tag-by-id`),
    /// an HTTP path, optionally after its method (`/markets/{id}`, `GET /tags/123`), a
    /// WebSocket channel URL or name, or an operation name (`getTeam`, `get-tag-by-id`,
    /// `GammaClient::get_tag`).
    pub(crate) query: String,
    /// Only match this HTTP method, e.g. `GET` or `POST`.
    #[serde(default)]
    pub(crate) method: Option<String>,
    /// Only match this service: `gamma`, `clob`, `data`, `relayer`, `bridge`, `combos` or `ws`.
    #[serde(default)]
    pub(crate) service: Option<String>,
}

/// Serves the marcasite docs.
#[derive(Debug, Clone)]
pub(crate) struct DocsServer {
    docs: Arc<Docs>,
    tool_router: ToolRouter<Self>,
}

impl DocsServer {
    pub(crate) fn new(docs: Arc<Docs>) -> Self {
        Self {
            docs,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl DocsServer {
    /// Ranked search over the marcasite public API.
    #[tool(
        name = "search",
        description = "Search the marcasite Rust SDK's public API (types, methods, modules) by \
                       name, path or doc text. Returns ranked `path (kind): summary` lines; \
                       pass a path to `get_item` for its full docs."
    )]
    fn search(&self, Parameters(p): Parameters<SearchParams>) -> Result<String, String> {
        let limit = p
            .limit
            .map_or(DEFAULT_LIMIT, |l| usize::try_from(l).unwrap_or(MAX_LIMIT))
            .clamp(1, MAX_LIMIT);
        self.docs
            .render_search(&p.query, p.module.as_deref(), p.kind.as_deref(), limit)
    }

    /// Full documentation of one item.
    #[tool(
        name = "get_item",
        description = "Get the full documentation of a marcasite item: signature or definition \
                       (fields, variants), docs, methods with signatures, cited Polymarket doc \
                       pages and related items. Accepts full or short paths \
                       (`GammaClient::list_markets`, `Event`); lists candidates when a short \
                       path is ambiguous."
    )]
    fn get_item(&self, Parameters(p): Parameters<GetItemParams>) -> Result<String, String> {
        self.docs.render_lookup(&p.path)
    }

    /// A module's overview.
    #[tool(
        name = "list_module",
        description = "List a marcasite module: its documentation (conventions, route tables) \
                       and an index of its items with one-line summaries, without item bodies. \
                       Modules: marcasite (crate root), types, gamma, clob, data, relayer, \
                       bridge, combos, ws."
    )]
    fn list_module(&self, Parameters(p): Parameters<ListModuleParams>) -> Result<String, String> {
        match self.docs.module(&p.module) {
            Some(m) => Ok(self.docs.render_module_index(m)),
            None => Err(format!(
                "No marcasite module is named `{}`. Modules: {}.",
                p.module,
                self.docs
                    .modules()
                    .iter()
                    .map(|m| format!("`{}`", Docs::module_id(m)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }

    /// Endpoint coverage.
    #[tool(
        name = "find_endpoint",
        description = "Find the marcasite method(s) and request builder(s) implementing a \
                       Polymarket API endpoint, given its doc URL, HTTP path (optionally with \
                       method and service) or operation name; or learn that it is not \
                       implemented in marcasite yet."
    )]
    fn find_endpoint(
        &self,
        Parameters(p): Parameters<FindEndpointParams>,
    ) -> Result<String, String> {
        self.docs
            .render_find_endpoint(&p.query, p.method.as_deref(), p.service.as_deref())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for DocsServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
        )
        .with_server_info(
            Implementation::new("marcasite-mcp", env!("CARGO_PKG_VERSION"))
                .with_title("marcasite SDK docs"),
        )
        .with_instructions(format!(
            "Documentation of marcasite {}, an unofficial Rust SDK for the Polymarket \
             Predictions APIs. Use it to write code against the SDK without guessing: `search` \
             finds items, `get_item` returns an item's signature and docs, `list_module` shows \
             what a module contains, and `find_endpoint` maps a Polymarket endpoint to the SDK \
             method implementing it (or says it is not implemented yet). Resources \
             `marcasite://sdk/{{module}}` hold whole modules and `marcasite://sdk/endpoints` the \
             endpoint coverage table; the `use-marcasite` prompt is a primer on the SDK's \
             idioms. This server only serves docs; it never calls Polymarket.",
            self.docs.version()
        ))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let mut resources: Vec<Resource> = self
            .docs
            .modules()
            .iter()
            .map(|m| {
                Resource::new(
                    format!("{MODULE_URI}{}", Docs::module_id(m)),
                    m.path.clone(),
                )
                .with_description(m.summary.clone())
                .with_mime_type(MARKDOWN)
            })
            .collect();
        resources.push(
            Resource::new(ENDPOINTS_URI, "endpoints")
                .with_title("marcasite endpoint coverage")
                .with_description(
                    "Every Polymarket Predictions API endpoint and the marcasite method \
                     implementing it, or that it is not implemented yet.",
                )
                .with_mime_type(MARKDOWN),
        );
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![
            ResourceTemplate::new(format!("{MODULE_URI}{{module}}"), "module")
                .with_title("marcasite module docs")
                .with_description(
                    "A marcasite module's full documentation, every item in full. `module` is \
                     `marcasite` (crate root), `types`, `gamma`, `clob`, `data`, `relayer`, \
                     `bridge`, `combos` or `ws`.",
                )
                .with_mime_type(MARKDOWN),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let uri = request.uri;
        let text = if uri == ENDPOINTS_URI {
            self.docs.render_endpoints()
        } else {
            match uri
                .strip_prefix(MODULE_URI)
                .and_then(|m| self.docs.module(m))
            {
                Some(m) => self.docs.render_module_full(m),
                None => {
                    return Err(ErrorData::resource_not_found(
                        format!("no marcasite docs resource at {uri}"),
                        None,
                    ));
                }
            }
        };
        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(text, uri).with_mime_type(MARKDOWN),
        ])
        .into())
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        Ok(ListPromptsResult::with_all_items(vec![Prompt::new(
            PROMPT,
            Some(
                "Primer on writing code with the marcasite SDK: client construction, request \
                 builders, pagination streams, errors, enums and value types, and the docs tools.",
            ),
            None,
        )]))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, ErrorData> {
        if request.name != PROMPT {
            return Err(ErrorData::invalid_params(
                format!(
                    "unknown prompt `{}`; the only prompt is `{PROMPT}`",
                    request.name
                ),
                None,
            ));
        }
        Ok(GetPromptResult::new(vec![PromptMessage::new_text(
            Role::User,
            self.docs.primer(),
        )])
        .with_description(format!("Primer on marcasite {}", self.docs.version()))
        .into())
    }
}
