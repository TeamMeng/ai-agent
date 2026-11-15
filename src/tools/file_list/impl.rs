use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    agent::ExecutionContext,
    tools::{file_list::execute::list, tool::Tool},
};

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListFilesArgs {
    #[serde(default = "default_path")]
    pub path: String,
}

fn default_path() -> String {
    ".".to_string()
}

pub struct ListFilesTool;

#[async_trait::async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "List files and directories at given path, directories listed first."
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(ListFilesArgs))
            .expect("schema is always serializable")
    }

    async fn execute(
        &self,
        args_json: &str,
        _context: &ExecutionContext,
    ) -> anyhow::Result<String> {
        let args: ListFilesArgs = serde_json::from_str(args_json)?;
        match list(&args.path) {
            Ok(listing) => Ok(listing),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}
