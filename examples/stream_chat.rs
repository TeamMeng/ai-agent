use ai_agent::llm::{complete_stream::chat_stream_with_retry, semaphore::get_semaphore};
use anyhow::Result;
use tokio::task::JoinSet;
use tracing::Instrument;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

const MODEL: &str = "deepseek-flash";

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let prompts = vec![
        "用三句话解释 Rust 的所有权机制",
        "什么是异步编程，和多线程有什么区别",
        "解释一下 TCP 三次握手的过程",
        "用简单的话说明什么是大语言模型",
        "Rust 中 Arc 和 Rc 的区别是什么",
        "什么是 RAG，为什么 AI 应用里常用它",
        "解释 HTTP 和 HTTPS 的区别",
        "什么是死锁，怎么避免",
        "用生活比喻解释什么是递归",
        "为什么说 Rust 没有 GC 但内存还是安全的",
    ];

    let mut set = JoinSet::new();
    for prompt in prompts {
        let span = tracing::info_span!("Chat", prompt = prompt);
        set.spawn(
            async move {
                tracing::info!("\n\n{prompt}");
                let permit = get_semaphore().acquire().await?;
                let output =
                    chat_stream_with_retry(MODEL, Some("你是一个全能助理"), prompt).await?;
                drop(permit);
                Ok::<_, anyhow::Error>((prompt, output))
            }
            .instrument(span),
        );
    }

    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok((prompt, result))) => tracing::info!("\n{}\n{}", prompt, result),
            Ok(Err(err)) => tracing::error!("Task panicked: {}", err),
            Err(err) => tracing::error!("Task panicked: {}", err),
        }
    }

    Ok(())
}
