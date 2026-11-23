use crate::agent::llm_request::LlmRequest;

pub mod compaction;
pub mod sliding_window;

pub fn count_tokens(_model: &str, request: &LlmRequest) -> usize {
    let bpe = tiktoken_rs::cl100k_base()
        .expect("Tiktoken should always resolve to the cl100k_base encoding");

    let mut total = 0;

    for instruction in &request.instructions {
        total += 4 + bpe.encode_ordinary(instruction).len();
    }

    for item in &request.contents {
        total += 4;
        total += match item {
            crate::agent::ContentItem::Message { content, .. } => {
                bpe.encode_ordinary(content).len()
            }
            crate::agent::ContentItem::ToolCall {
                name, arguments, ..
            } => {
                bpe.encode_ordinary(name).len() + bpe.encode_ordinary(&arguments.to_string()).len()
            }
            crate::agent::ContentItem::ToolResult { content, .. } => {
                bpe.encode_ordinary(content).len()
            }
        }
    }

    total
}
