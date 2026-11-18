use std::collections::HashSet;

use crate::{
    agent::{
        ContentItem, ExecutionContext,
        llm_request::{BeforeLlmCallback, LlmRequest},
    },
    callback::context_optimizer::count_tokens,
};

pub struct SlidingWindow {
    pub model: String,
    pub token_threshold: usize,
    pub window_size: usize,
}

#[async_trait::async_trait]
impl BeforeLlmCallback for SlidingWindow {
    async fn call(&self, _context: &mut ExecutionContext, request: &mut LlmRequest) {
        if count_tokens(&self.model, request) < self.token_threshold {
            return;
        }

        if request.contents.len() <= self.window_size {
            return;
        }

        let user_idx = request
            .contents
            .iter()
            .position(|item| matches!(item, ContentItem::Message { role, .. } if role == "user"));
        let Some(user_idx) = user_idx else { return };

        let start = find_safe_start(&request.contents, request.contents.len() - self.window_size);
        let start = start.max(user_idx + 1);

        request.contents.drain(user_idx + 1..start);
    }
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

#[cfg(test)]
mod tests {
    use crate::agent::ToolResultStatus;

    use super::*;

    fn message(role: &str, content: &str) -> ContentItem {
        ContentItem::Message {
            role: role.into(),
            content: content.into(),
        }
    }

    fn tool_call(id: &str) -> ContentItem {
        ContentItem::ToolCall {
            tool_call_id: id.into(),
            name: "lookup".into(),
            arguments: serde_json::json!({}),
        }
    }

    fn tool_result(id: &str) -> ContentItem {
        ContentItem::ToolResult {
            tool_call_id: id.into(),
            name: "lookup".into(),
            status: ToolResultStatus::Success,
            content: "ok".into(),
        }
    }

    fn sliding_window(window_size: usize) -> SlidingWindow {
        SlidingWindow {
            model: "test".into(),
            token_threshold: 0,
            window_size,
        }
    }

    #[tokio::test]
    async fn does_not_split_a_tool_call_from_its_result() {
        let mut request = LlmRequest {
            instructions: Vec::new(),
            contents: vec![
                message("user", "task"),
                message("assistant", "working"),
                message("assistant", "still working"),
                tool_call("call_1"),
                tool_result("call_1"),
                message("assistant", "done"),
            ],
        };

        sliding_window(2)
            .call(&mut ExecutionContext::new(), &mut request)
            .await;

        assert_eq!(
            serde_json::to_value(&request.contents).expect("serialize contents"),
            serde_json::json!([
                {"type": "message", "role": "user", "content": "task"},
                {"type": "tool_call", "tool_call_id": "call_1", "name": "lookup", "arguments": {}},
                {"type": "tool_result", "tool_call_id": "call_1", "name": "lookup", "status": "success", "content": "ok"},
                {"type": "message", "role": "assistant", "content": "done"}
            ])
        );
    }

    #[tokio::test]
    async fn leaves_context_shorter_than_window_unchanged() {
        let mut request = LlmRequest {
            instructions: Vec::new(),
            contents: vec![message("user", "task"), tool_call("call_1")],
        };
        let before = serde_json::to_value(&request.contents).expect("serialize contents");

        sliding_window(2)
            .call(&mut ExecutionContext::new(), &mut request)
            .await;

        assert_eq!(
            serde_json::to_value(&request.contents).expect("serialize contents"),
            before
        );
    }
}
