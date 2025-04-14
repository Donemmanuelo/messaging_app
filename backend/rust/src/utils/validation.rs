use validator::Validate;
use serde::Deserialize;

pub fn validate_request<T: Validate>(request: &T) -> Result<(), String> {
    if let Err(errors) = request.validate() {
        let error_messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(_, errors)| errors.iter().map(|e| e.message.clone().unwrap_or_default()))
            .collect();
        return Err(error_messages.join(", "));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 100))]
    pub page: Option<u32>,
    #[validate(range(min = 1, max = 50))]
    pub per_page: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
        }
    }
} 