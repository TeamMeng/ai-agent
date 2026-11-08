use ai_agent::llm::{complete::chat_complete, structured::chat_complete_structured};
use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const MODEL: &str = "deepseek-flash";

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let content = chat_complete(MODEL, Some("你是一个全能的助手"), "爱尔兰的首都在哪里").await?;

    println!("Response: {}", content);

    let plan_content =
        chat_complete_structured(MODEL, "我要去参加美加墨世界杯观看比赛，如果安排？").await?;

    println!("Plan Response: {:?}", plan_content);

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
