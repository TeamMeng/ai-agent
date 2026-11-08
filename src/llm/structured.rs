use async_openai::types::chat::{
    ChatCompletionMessageToolCalls, ChatCompletionNamedToolChoice,
    ChatCompletionRequestUserMessageArgs, ChatCompletionTool, ChatCompletionToolChoiceOption,
    ChatCompletionTools, CreateChatCompletionRequestArgs, FunctionName, FunctionObject,
    ReasoningEffort,
};

use crate::models::action_plan::ActionPlan;

/// Name of the function the model is forced to call.
const TOOL_NAME: &str = "emit_action_plan";

pub async fn chat_complete_structured(model: &str, prompt: &str) -> anyhow::Result<ActionPlan> {
    let client = async_openai::Client::new();

    let messages = vec![
        ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into(),
    ];

    // DeepSeek's chat API does not support `response_format: json_schema`, but
    // it does support tool calling, so the JSON schema travels as the function
    // parameters and the model is forced to call it.
    let schema = schemars::schema_for!(ActionPlan);
    let tool = ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObject {
            name: TOOL_NAME.into(),
            description: Some(
                "Emit a step-by-step agent action plan with difficulty and time estimate".into(),
            ),
            parameters: Some(schema.as_value().clone()),
            strict: None,
        },
    });

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .tools(vec![tool])
        .tool_choice(ChatCompletionToolChoiceOption::Function(
            ChatCompletionNamedToolChoice {
                function: FunctionName {
                    name: TOOL_NAME.into(),
                },
            },
        ))
        // DeepSeek rejects forced tool choices while thinking mode is active,
        // and thinking is on by default.
        .reasoning_effort(ReasoningEffort::None)
        .max_tokens(2048u32)
        .build()?;

    let response = client.chat().create(request).await?;

    let arguments = response
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.tool_calls)
        .and_then(|calls| {
            calls.into_iter().find_map(|call| match call {
                ChatCompletionMessageToolCalls::Function(call) => Some(call.function.arguments),
                _ => None,
            })
        })
        .ok_or_else(|| anyhow::anyhow!("No function call in response"))?;

    let plan = serde_json::from_str(&arguments)?;

    Ok(plan)
}
