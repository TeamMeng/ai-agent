use ai_agent::{llm::complete::chat_complete, tools::tools};
use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const MODEL: &str = "deepseek-flash";

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let tools = tools();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let content = chat_complete(
        MODEL,
        Some("你是一个全能的助手"),
        "5876乘以675是多少？",
        tools,
    )
    .await?;

    println!("Response: {}", content);

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
