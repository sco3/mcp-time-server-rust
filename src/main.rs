use rmcp::tool_router;
use rmcp::tool;
use rmcp::tool_handler;
use rmcp::model::{CallToolResult, Content, ServerInfo, ServerCapabilities, Implementation, ProtocolVersion};
use rmcp::ErrorData as McpError;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, StreamableHttpServerConfig,
};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::ServerHandler;
use rmcp::handler::server::tool::ToolRouter;
use tokio_util::sync::CancellationToken;

const BIND_ADDRESS: &str = "127.0.0.1:3000";

pub struct TimeServer {
    tool_router: ToolRouter<TimeServer>,
}

#[tool_router]
impl TimeServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Time server")]
    async fn get_time(&self) -> Result<CallToolResult, McpError> {
        let current_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Ok(CallToolResult::success(vec![Content::text(current_time)]))
    }
}

#[tool_handler]
impl ServerHandler for TimeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "time-server".to_string(),
                version: "0.1.0".to_string(),
                description: None,
                title: None,
                website_url: None,
                icons: None,
            },
            instructions: Some("A simple time server that returns the current time".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ct = CancellationToken::new();

    let service = StreamableHttpService::new(
        || Ok(TimeServer::new()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig {
            stateful_mode: false,
            cancellation_token: ct.child_token(),
            ..Default::default()
        },
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;
    let _ = axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.unwrap();
            ct.cancel();
        })
        .await;
    Ok(())
}
