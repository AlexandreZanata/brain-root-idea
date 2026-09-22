//! Shared OpenCode Go failure classification.
//!
//! One place turns HTTP status and transport conditions into the frozen
//! provider-neutral [`ErrorCode`]s with BrainRoot's plain-language messages.
//! Classification never reads a response body and never surfaces provider
//! text, and it never substitutes a fallback model, endpoint, or balance.

use super::contract::ErrorCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoFailure {
    InvalidKey { status: u16 },
    SubscriptionUnavailable { status: u16 },
    UnsupportedModel { status: u16 },
    UsageLimitReached { status: u16 },
    Timeout,
    NetworkUnavailable,
    ProviderFailure { status: u16 },
    MalformedResponse,
}

impl GoFailure {
    /// Classifies a non-success HTTP status. Statuses are externally mutable;
    /// unknown values stay visible as provider failures with their number.
    pub fn from_status(status: u16) -> Self {
        match status {
            401 => Self::InvalidKey { status },
            402 | 403 => Self::SubscriptionUnavailable { status },
            404 => Self::UnsupportedModel { status },
            429 => Self::UsageLimitReached { status },
            _ => Self::ProviderFailure { status },
        }
    }

    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidKey { .. } | Self::SubscriptionUnavailable { .. } => {
                ErrorCode::AuthenticationFailed
            }
            Self::UnsupportedModel { .. } => ErrorCode::InvalidInput,
            Self::UsageLimitReached { .. } => ErrorCode::RateLimited,
            Self::Timeout => ErrorCode::TimedOut,
            Self::NetworkUnavailable | Self::ProviderFailure { .. } => {
                ErrorCode::ProviderUnavailable
            }
            Self::MalformedResponse => ErrorCode::MalformedResponse,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::InvalidKey { .. } => {
                "The provider rejected the credential. Check your OpenCode Go key and try again."
                    .to_string()
            }
            Self::SubscriptionUnavailable { .. } => {
                "This OpenCode Go subscription is not active for this account. Check the subscription in the OpenCode Zen console."
                    .to_string()
            }
            Self::UnsupportedModel { .. } => {
                "The selected model is not available on OpenCode Go. Choose a different model and try again."
                    .to_string()
            }
            Self::UsageLimitReached { .. } => {
                "The OpenCode Go usage limit was reached for this period. Try again later or choose another model."
                    .to_string()
            }
            Self::Timeout => "The provider took too long to respond. Try again.".to_string(),
            Self::NetworkUnavailable => {
                "The provider could not be reached. Check your connection and try again.".to_string()
            }
            Self::ProviderFailure { status } if (500..=599).contains(status) => {
                "The provider is temporarily unavailable. Try again shortly.".to_string()
            }
            Self::ProviderFailure { status } => {
                format!("The provider answered with status {status}. Try again.")
            }
            Self::MalformedResponse => {
                "The provider returned an unreadable response. Try again.".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One representative per user-visible outcome; `402`/`403` share a case.
    fn distinct_cases() -> Vec<GoFailure> {
        vec![
            GoFailure::InvalidKey { status: 401 },
            GoFailure::SubscriptionUnavailable { status: 403 },
            GoFailure::UnsupportedModel { status: 404 },
            GoFailure::UsageLimitReached { status: 429 },
            GoFailure::ProviderFailure { status: 500 },
            GoFailure::ProviderFailure { status: 418 },
            GoFailure::Timeout,
            GoFailure::NetworkUnavailable,
            GoFailure::MalformedResponse,
        ]
    }

    #[test]
    fn status_table_matches_the_documented_mapping() {
        assert_eq!(
            GoFailure::from_status(401),
            GoFailure::InvalidKey { status: 401 }
        );
        assert_eq!(
            GoFailure::from_status(402),
            GoFailure::SubscriptionUnavailable { status: 402 }
        );
        assert_eq!(
            GoFailure::from_status(403),
            GoFailure::SubscriptionUnavailable { status: 403 }
        );
        assert_eq!(
            GoFailure::from_status(404),
            GoFailure::UnsupportedModel { status: 404 }
        );
        assert_eq!(
            GoFailure::from_status(429),
            GoFailure::UsageLimitReached { status: 429 }
        );
    }

    #[test]
    fn server_and_unknown_statuses_stay_provider_failures() {
        for status in [400, 418, 500, 503, 599] {
            assert_eq!(
                GoFailure::from_status(status),
                GoFailure::ProviderFailure { status }
            );
        }
    }

    #[test]
    fn each_case_uses_an_existing_neutral_code() {
        assert_eq!(
            GoFailure::from_status(401).code(),
            ErrorCode::AuthenticationFailed
        );
        assert_eq!(
            GoFailure::from_status(403).code(),
            ErrorCode::AuthenticationFailed
        );
        assert_eq!(GoFailure::from_status(404).code(), ErrorCode::InvalidInput);
        assert_eq!(GoFailure::from_status(429).code(), ErrorCode::RateLimited);
        assert_eq!(
            GoFailure::from_status(500).code(),
            ErrorCode::ProviderUnavailable
        );
        assert_eq!(GoFailure::Timeout.code(), ErrorCode::TimedOut);
        assert_eq!(
            GoFailure::NetworkUnavailable.code(),
            ErrorCode::ProviderUnavailable
        );
        assert_eq!(
            GoFailure::MalformedResponse.code(),
            ErrorCode::MalformedResponse
        );
    }

    #[test]
    fn messages_are_plain_language_and_distinct() {
        let messages: Vec<String> = distinct_cases().iter().map(GoFailure::message).collect();

        assert!(messages.iter().all(|message| !message.is_empty()));
        let mut unique = messages.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), messages.len(), "{messages:?}");

        assert!(messages[0].contains("rejected the credential"));
        assert!(messages[1].contains("subscription"));
        assert!(messages[2].contains("not available"));
        assert!(messages[3].contains("usage limit"));
        assert!(messages[4].contains("temporarily unavailable"));
        assert!(messages[5].contains("418"));

        assert_eq!(
            GoFailure::from_status(402).message(),
            GoFailure::from_status(403).message()
        );
    }
}
