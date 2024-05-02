use crate::{
    constants::{BASE_URL, MODEL_ID},
    types::{APIResponse, ChatCompletionRequestMessage},
    AppError,
};
use reqwest::Client as HttpClient;
use serde_json::json;

pub async fn ChatCompletion(
    messages: Vec<ChatCompletionRequestMessage>,
) -> Result<String, AppError> {
    let r_client = HttpClient::new();
    let payload = json!({
        "messages": messages,
        "model_id": MODEL_ID,
    });
    let request = r_client
        .post(BASE_URL.to_owned() + "/models")
        .json(&payload)
        .send()
        .await
        .map_err(AppError::ReqwestError)?;
    let response = request.json::<APIResponse>().await?;
    let content = response
        .content
        .unwrap_or_else(|| "I can't respond to that.".to_owned());
    Ok(content)
}
