use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct LlmToolResult {
    #[serde(rename = "type")]
    pub(crate) result_type: String,
    pub(crate) tool_use_id: String,
    pub(crate) content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct LlmToolTextRequest {
    #[serde(rename = "type")]
    pub(crate) request_type: String,
    pub(crate) text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct LlmToolUseRequest {
    #[serde(rename = "type")]
    pub(crate) request_type: String,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) input: serde_json::value::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged, rename_all = "snake_case")]
pub(crate) enum LlmToolRequest {
    LlmToolTextRequest(LlmToolTextRequest),
    LlmToolUseRequest(LlmToolUseRequest),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GetWeatherToolInput {
    pub(crate) location: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GetRestaurantsToolInput {
    pub(crate) location: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub(crate) enum LlmToolInput {
    GetWeather(GetWeatherToolInput),
    GetRestaurantsToolInput(GetRestaurantsToolInput),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct LlmTaskResult {
    pub(crate) role: String,
    pub(crate) content: Vec<LlmToolRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct EventInput {
    pub(crate) messages: Vec<serde_json::Value>,
    #[serde(rename = "taskResult")]
    pub(crate) task_result: LlmTaskResult,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct LlmToolResultAnswer {
    pub(crate) role: String,
    pub(crate) content: Vec<LlmToolResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum ResultMessage {
    MessageValue(serde_json::Value),
    LlmTaskResult(LlmTaskResult),
    LlmToolResultAnswer(LlmToolResultAnswer),
}

pub(crate) fn handle_input(req_full: EventInput) -> Vec<ResultMessage> {
    // get previous messages from the dialog with LLM
    // they are needed only to be passed further as a chain of messages
    // so can be treated as vector of serde_json::Value and simple passed with result
    let context_messages: Vec<ResultMessage> = req_full
        .messages
        .into_iter()
        .map(|msg| ResultMessage::MessageValue(msg))
        .collect();

    // get "tool_use" messages from task result passed to the tunction
    let current_use_tools = req_full.task_result.content.iter().fold(
        vec![],
        |mut acc, tool_request| match tool_request {
            LlmToolRequest::LlmToolUseRequest(req_use) => {
                acc.push(req_use);
                acc
            }
            _ => acc,
        },
    );

    // because different tools might have the same input structure
    // I am matching on "type" in the request from assistant

    let tool_result: Vec<LlmToolResult> = current_use_tools
        .into_iter()
        .map(|req_use| {
            let input = req_use.input.clone();

            let tool_result = match req_use.name.as_str() {
                "get_weather" => {
                    let inp = serde_json::from_value::<GetWeatherToolInput>(input).unwrap();
                    get_weather(req_use.id.clone(), inp)
                }
                "get_restaurants" => {
                    let inp = serde_json::from_value::<GetRestaurantsToolInput>(input).unwrap();
                    get_restaurants(req_use.id.clone(), inp)
                }
                _ => panic!("unknown tool name"),
            };

            tool_result
        })
        .collect::<Vec<_>>();

    let tools_answer = LlmToolResultAnswer {
        role: String::from("user"),
        content: tool_result,
    };

    let mut result_messages: Vec<ResultMessage> = context_messages;

    result_messages.push(ResultMessage::LlmTaskResult(req_full.task_result));

    result_messages.push(ResultMessage::LlmToolResultAnswer(tools_answer));
    result_messages
}

pub(crate) fn get_weather(tool_use_id: String, input: GetWeatherToolInput) -> LlmToolResult {
    println!("checking weather for {}", input.location);
    LlmToolResult {
        result_type: String::from("tool_result"),
        tool_use_id,
        content: String::from("The weather is sunny, 20 degree"),
    }
}

pub(crate) fn get_restaurants(
    tool_use_id: String,
    input: GetRestaurantsToolInput,
) -> LlmToolResult {
    println!("checking restaurants for {}", input.location);
    LlmToolResult {
        result_type: String::from("tool_result"),
        tool_use_id,
        content: String::from(
            r#"
<restaurants>
            <restaurant>
                <name>Restaurant ABC</name>
                <address>Street 111</address>
                <phone>12345678</phone>
                <website>www.restaurant1.com</website>
                <cuisine>Italian</cuisine>
                <outdoor>true</outdoor>
            </restaurant>
            <restaurant>
                <name>Restaurant XYZ</name>
                <address>Street 999</address>
                <phone>987654</phone>
                <website>www.restaurant2.com</website>
                <cuisine>French</cuisine>
                <outdoor>false</outdoor>
            </restaurant>
</restaurants>
"#,
        ),
    }
}
