use ai_agent::{llm::complete::chat_complete, tools::build_toolbox};
use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const MODEL: &str = "deepseek-flash";

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let toolbox = build_toolbox();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let content = chat_complete(
        MODEL,
        Some("你是一个全能的助手"),
        "5876乘以675是多少？",
        &toolbox,
    )
    .await?;

    println!("Response: {}", content);

    Ok(())
}
