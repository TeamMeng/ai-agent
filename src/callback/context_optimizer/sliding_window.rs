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

        let user_idx = request
            .contents
            .iter()
            .position(|item| matches!(item, ContentItem::Message { role, ..} if role == "user"));
        let Some(user_idx) = user_idx else { return };
        let preserved = request.contents[..=user_idx].to_vec();

        let mut remaining = request.contents[user_idx + 1..].to_vec();
        if remaining.len() > self.window_size {
            let cut = remaining.len() - self.window_size;
            remaining = remaining.split_off(cut);
        }

        request.contents = preserved.into_iter().chain(remaining).collect();
    }
}
