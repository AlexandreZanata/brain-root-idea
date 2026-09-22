//! Versioned conversation IPC wire types.

use serde::{Deserialize, Serialize};

use crate::provider::contract::{ConversationId, StreamEvent};

pub const CONVERSATION_EVENT_NAME: &str = "conversation_event";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ConversationSendRequest {
    pub contract_version: u32,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ConversationAccepted {
    pub contract_version: u32,
    pub conversation: ConversationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ConversationEnvelope {
    pub contract_version: u32,
    pub conversation: ConversationId,
    pub event: StreamEvent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::contract::{ConversationId, StreamEvent, PROVIDER_CONTRACT_VERSION};

    #[test]
    fn envelope_serialization_is_versioned_and_provider_neutral() {
        let envelope = ConversationEnvelope {
            contract_version: PROVIDER_CONTRACT_VERSION,
            conversation: ConversationId::new("conversation-test").expect("valid"),
            event: StreamEvent::TextChunk {
                text: "safe text".to_string(),
            },
        };
        let value = serde_json::to_value(envelope).expect("serializes");

        assert_eq!(value["contractVersion"], PROVIDER_CONTRACT_VERSION);
        assert_eq!(value["conversation"], "conversation-test");
        assert_eq!(value["event"]["type"], "text_chunk");
        assert_eq!(value["event"]["text"], "safe text");
        let rendered = value.to_string();
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("credential"));
        assert!(!rendered.contains("opencode"));
    }
}
