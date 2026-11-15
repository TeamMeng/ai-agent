use std::sync::Arc;

use ai_agent::{agent::Agent, tools::build_file_explorer_toolbox};
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

    let toolbox = Arc::new(build_file_explorer_toolbox(MODEL));

    let instructions = r#"你是一位善于探索文件的助手。
拿到一个压缩包时，先解压，再用 list_files 看目录结构，
靠文件名判断哪些文件可能相关，用 read_file 或 read_image 逐一确认，
最后再给出结论。不要跳过探索步骤直接猜答案。"#;

    let agent = Agent::new(MODEL, Some(instructions), &toolbox).with_max_steps(20);

    let result = agent.run("查看当前目录有哪些文件?").await?;

    println!("Result: {}", result.output);
    println!(
        "All steps: {}, events: {}",
        result.context.current_step,
        result.context.events.len()
    );

    Ok(())
}
