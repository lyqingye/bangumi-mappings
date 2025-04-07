use rig::{completion::ToolDefinition, tool::Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SubmitArgs {
    id: Option<i32>,
    name: Option<String>,
    season: Option<i32>,
    confidence_score: Option<i32>,
}

pub struct SubmitTool {}

impl SubmitTool {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, thiserror::Error, Serialize)]
#[error("{message}")]
pub struct SubmitError {
    message: String,
}

impl Tool for SubmitTool {
    const NAME: &'static str = "submit";

    type Error = SubmitError;
    type Args = SubmitArgs;
    type Output = ();

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "submit".to_string(),
            description: "Submit the match result".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "number",
                        "description": "The id of the anime"
                    },
                    "name": {
                        "type": "string",
                        "description": "The name of the anime"
                    },
                    "season": {
                        "type": "number",
                        "description": "The season number of the anime"
                    },
                    "confidence_score": {
                        "type": "number",
                        "description": "The confidence score of the match, value range from 0 to 100"
                    },
                },
                "required": ["confidence_score"]
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

pub struct SubmitBGMTool {}

impl Tool for SubmitBGMTool {
    const NAME: &'static str = "submit";

    type Error = SubmitError;
    type Args = SubmitArgs;
    type Output = ();

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "submit".to_string(),
            description: "Submit the match result".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "number",
                        "description": "The id of the anime"
                    },
                    "name": {
                        "type": "string",
                        "description": "The name of the anime"
                    },
                    "confidence_score": {
                        "type": "number",
                        "description": "The confidence score of the match, value range from 0 to 100"
                    },
                },
                "required": ["confidence_score"]
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
