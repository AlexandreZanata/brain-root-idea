use serde::{Deserialize, Serialize};

pub const HEALTH_CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthRequest {
    pub contract_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub contract_version: u32,
    pub status: HealthStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum HealthError {
    UnsupportedContractVersion { supported: u32, received: u32 },
}

#[tauri::command]
pub fn health(request: HealthRequest) -> Result<HealthReport, HealthError> {
    if request.contract_version != HEALTH_CONTRACT_VERSION {
        return Err(HealthError::UnsupportedContractVersion {
            supported: HEALTH_CONTRACT_VERSION,
            received: request.contract_version,
        });
    }

    Ok(HealthReport {
        contract_version: HEALTH_CONTRACT_VERSION,
        status: HealthStatus::Ready,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_request_returns_ready_report() {
        let report = health(HealthRequest {
            contract_version: HEALTH_CONTRACT_VERSION,
        })
        .expect("valid request");

        assert_eq!(
            report,
            HealthReport {
                contract_version: HEALTH_CONTRACT_VERSION,
                status: HealthStatus::Ready,
            }
        );
    }

    #[test]
    fn malformed_request_json_is_rejected() {
        let error = serde_json::from_str::<HealthRequest>("{\"contract_version\": \"one\"}")
            .expect_err("a string version must be rejected");

        assert!(
            error.to_string().contains("invalid type"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn unknown_field_is_rejected() {
        serde_json::from_str::<HealthRequest>("{\"contract_version\": 1, \"extra\": true}")
            .expect_err("unknown fields must be rejected");
    }

    #[test]
    fn missing_field_is_rejected() {
        serde_json::from_str::<HealthRequest>("{}").expect_err("a missing field must be rejected");
    }

    #[test]
    fn unsupported_version_returns_typed_error() {
        let error = health(HealthRequest {
            contract_version: 99,
        })
        .expect_err("an unsupported version must fail");

        assert_eq!(
            error,
            HealthError::UnsupportedContractVersion {
                supported: HEALTH_CONTRACT_VERSION,
                received: 99,
            }
        );
    }

    #[test]
    fn report_serializes_with_typed_keys_only() {
        let value = serde_json::to_value(HealthReport {
            contract_version: HEALTH_CONTRACT_VERSION,
            status: HealthStatus::Ready,
        })
        .expect("the report serializes");

        let object = value.as_object().expect("the report is an object");
        let mut keys: Vec<&String> = object.keys().collect();
        keys.sort();

        assert_eq!(keys, vec!["contract_version", "status"]);
        assert_eq!(object.get("status"), Some(&serde_json::json!("ready")));
    }
}
