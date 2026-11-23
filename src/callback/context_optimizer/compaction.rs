use std::collections::HashMap;

use serde_json::Value;

use crate::agent::{ContentItem, llm_request::LlmRequest};

pub struct Compaction {
    pub keep_recent: usize,
}

impl Compaction {
    pub fn apply(&self, request: &mut LlmRequest) {
        let protect_from = request.contents.len().saturating_sub(self.keep_recent);
        let mut call_args: HashMap<String, Value> = HashMap::new();

        for (idx, item) in request.contents.iter_mut().enumerate() {
            match item {
                ContentItem::ToolCall {
                    tool_call_id,
                    arguments,
                    ..
                } => {
                    call_args.insert(tool_call_id.clone(), arguments.clone());
                }
                ContentItem::ToolResult {
                    tool_call_id,
                    name,
                    content,
                    ..
                } => {
                    if idx >= protect_from {
                        continue;
                    }

                    let args = call_args.get(tool_call_id);
                    let replacement = match name.as_str() {
                        "read_file" => {
                            let path = args
                                .and_then(|a| a.get("file_path"))
                                .and_then(Value::as_str)
                                .unwrap_or("unknown");
                            Some(format!(
                                "File '{path}' was already read. Call read_file again if you need it."
                            ))
                        }
                        _ => None,
                    };

                    if let Some(replacement) = replacement {
                        *content = replacement;
                    }
                }
                _ => {}
            }
        }
    }
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

    fn tool_call(id: &str, name: &str, arguments: Value) -> ContentItem {
        ContentItem::ToolCall {
            tool_call_id: id.into(),
            name: name.into(),
            arguments,
        }
    }

    fn tool_result(id: &str, name: &str, content: &str) -> ContentItem {
        ContentItem::ToolResult {
            tool_call_id: id.into(),
            name: name.into(),
            status: ToolResultStatus::Success,
            content: content.into(),
        }
    }

    fn request(contents: Vec<ContentItem>) -> LlmRequest {
        LlmRequest {
            instructions: Vec::new(),
            contents,
        }
    }

    #[test]
    fn replaces_old_read_file_results_with_a_pointer() {
        let mut request = request(vec![
            message("user", "task"),
            tool_call(
                "call_1",
                "read_file",
                serde_json::json!({"file_path": "src/main.rs"}),
            ),
            tool_result("call_1", "read_file", "line 1\nline 2"),
            tool_call("call_2", "lookup", serde_json::json!({})),
            tool_result("call_2", "lookup", "ok"),
        ]);

        Compaction { keep_recent: 2 }.apply(&mut request);

        assert_eq!(
            request.contents[2],
            tool_result(
                "call_1",
                "read_file",
                "File 'src/main.rs' was already read. Call read_file again if you need it."
            )
        );
        assert_eq!(request.contents[4], tool_result("call_2", "lookup", "ok"));
    }

    #[test]
    fn leaves_results_within_the_recent_window_unchanged() {
        let mut request = request(vec![
            tool_call(
                "call_1",
                "read_file",
                serde_json::json!({"file_path": "a.txt"}),
            ),
            tool_result("call_1", "read_file", "contents"),
        ]);
        let before = request.contents.clone();

        Compaction { keep_recent: 2 }.apply(&mut request);

        assert_eq!(request.contents, before);
    }

    #[test]
    fn falls_back_to_unknown_path_when_arguments_are_missing() {
        let mut request = request(vec![
            tool_result("call_1", "read_file", "contents"),
            message("user", "task"),
        ]);

        Compaction { keep_recent: 1 }.apply(&mut request);

        assert_eq!(
            request.contents[0],
            tool_result(
                "call_1",
                "read_file",
                "File 'unknown' was already read. Call read_file again if you need it."
            )
        );
    }
}
