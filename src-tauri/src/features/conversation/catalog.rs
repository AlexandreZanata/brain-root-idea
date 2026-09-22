//! Model catalog selection for the conversation feature.

use crate::provider::contract::ModelId;
use crate::provider::discovery::{DiscoveredModel, ProtocolEndpoint};
use crate::provider::go::DEFAULT_MODEL_ID;

pub(super) fn select_default_model(models: &[DiscoveredModel]) -> Option<ModelId> {
    models
        .iter()
        .find(|model| {
            model.id.as_str() == DEFAULT_MODEL_ID
                && matches!(
                    model.endpoint,
                    None | Some(ProtocolEndpoint::ChatCompletions)
                )
        })
        .map(|model| model.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::discovery::ModelPrivacy;

    fn model(id: &str, endpoint: Option<ProtocolEndpoint>) -> DiscoveredModel {
        DiscoveredModel {
            id: ModelId::new(id).expect("valid model"),
            display_name: id.to_string(),
            endpoint,
            privacy: ModelPrivacy::unknown(),
        }
    }

    #[test]
    fn default_model_requires_exact_id_and_accepts_an_unstated_endpoint() {
        let catalog = vec![
            model(DEFAULT_MODEL_ID, Some(ProtocolEndpoint::Messages)),
            model("another-model", Some(ProtocolEndpoint::ChatCompletions)),
        ];
        assert_eq!(select_default_model(&catalog), None);

        let with_unstated = vec![model(DEFAULT_MODEL_ID, None)];
        assert_eq!(
            select_default_model(&with_unstated)
                .expect("an unstated endpoint is a candidate")
                .as_str(),
            DEFAULT_MODEL_ID
        );

        let with_chat = vec![model(
            DEFAULT_MODEL_ID,
            Some(ProtocolEndpoint::ChatCompletions),
        )];
        assert_eq!(
            select_default_model(&with_chat)
                .expect("the chat endpoint is compatible")
                .as_str(),
            DEFAULT_MODEL_ID
        );
    }
}
