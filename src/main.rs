use ai_agent::llm::complete::chat_complete;
use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let content = chat_complete(
        "deepseek-flash",
        Some("你是一个全能的助手"),
        "爱尔兰的首都在哪里",
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
