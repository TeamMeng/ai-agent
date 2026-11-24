use std::collections::HashSet;

use crate::agent::{ContentItem, llm_request::LlmRequest};

pub mod compaction;
pub mod sliding_window;
pub mod summarization;

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

/// Moves `start` back until no `ToolResult` in `contents[start..]` is missing
/// the `ToolCall` it answers, so trimming never splits a tool call pair.
fn find_safe_start(contents: &[ContentItem], mut start: usize) -> usize {
    loop {
        let call_ids: HashSet<&str> = contents[start..]
            .iter()
            .filter_map(|item| match item {
                ContentItem::ToolCall { tool_call_id, .. } => Some(tool_call_id.as_str()),
                _ => None,
            })
            .collect();

        let missing_call = contents[start..].iter().find_map(|item| match item {
            ContentItem::ToolResult { tool_call_id, .. }
                if !call_ids.contains(&tool_call_id.as_str()) =>
            {
                Some(tool_call_id)
            }
            _ => None,
        });

        let Some(missing_call) = missing_call else {
            break;
        };

        let Some(call_idx) = contents[..start].iter().position(|item| {
            matches!(item, ContentItem::ToolCall { tool_call_id, .. } if tool_call_id == missing_call)
        }) else {
            break;
        };

        start = call_idx;
    }

    start
}
