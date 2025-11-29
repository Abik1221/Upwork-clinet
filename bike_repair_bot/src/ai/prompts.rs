use crate::models::Message;

/// System prompt for the motorcycle repair assistant
pub const SYSTEM_PROMPT: &str = r#"You are an expert motorcycle mechanic and repair assistant with decades of experience. Your role is to help users diagnose and fix motorcycle issues.

**Guidelines:**
- Only answer questions related to motorcycle repair, maintenance, diagnosis, and parts
- Base your answers on the provided manual context when available
- If the manual doesn't contain relevant information, use your general motorcycle knowledge
- Always prioritize safety - warn users about dangerous procedures
- Provide clear, step-by-step instructions when appropriate
- Ask clarifying questions if the user's query is ambiguous (bike model, symptoms, etc.)
- Suggest professional help for complex, dangerous, or safety-critical repairs
- If a question is not about motorcycles, politely decline and remind users of your purpose
- Be concise but thorough
- Use bullet points and numbered lists for clarity

**Safety Rules:**
- Always recommend safety gear (gloves, goggles, etc.)
- Warn about hot engine parts, high voltage, compressed springs, etc.
- Recommend proper tools for the job
- Suggest torque specs when relevant
- Warn about fuel, oil, and chemical hazards

When citing manual information, always mention the source (e.g., "According to the manual...").
"#;

/// Build the complete prompt for a chat request
pub fn build_chat_prompt(
    user_query: &str,
    retrieved_context: Option<&str>,
    chat_history: &[Message],
) -> Vec<Message> {
    let mut messages = Vec::new();

    // System prompt with context if available
    let system_content = if let Some(context) = retrieved_context {
        format!(
            "{}\n\n**Manual Context:**\n{}\n\nAlways cite the manual when using this context.",
            SYSTEM_PROMPT, context
        )
    } else {
        SYSTEM_PROMPT.to_string()
    };

    messages.push(Message::system(system_content));

    // Add recent chat history (limit to last 6 messages to avoid token limits)
    let recent_history = if chat_history.len() > 6 {
        &chat_history[chat_history.len() - 6..]
    } else {
        chat_history
    };

    messages.extend(recent_history.iter().cloned());

    // Add current user query
    messages.push(Message::user(user_query));

    messages
}

/// Validate that a response is appropriate
pub fn validate_response(response: &str) -> bool {
    // Make sure response isn't empty
    if response.trim().is_empty() {
        return false;
    }

    // Make sure response isn't too long (sanity check)
    if response.len() > 10000 {
        log::warn!("Response too long: {} chars", response.len());
        return false;
    }

    // Could add more validation here (profanity filter, etc.)
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_chat_prompt() {
        let messages = build_chat_prompt(
            "How do I change oil?",
            Some("Oil change procedure:\n1. Warm engine\n2. Drain oil"),
            &[],
        );

        assert_eq!(messages.len(), 2); // system + user
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[1].role, "user");
        assert!(messages[0].content.contains("Manual Context"));
    }

    #[test]
    fn test_validate_response() {
        assert!(validate_response("This is a valid response"));
        assert!(!validate_response(""));
        assert!(!validate_response("   "));
    }
}
