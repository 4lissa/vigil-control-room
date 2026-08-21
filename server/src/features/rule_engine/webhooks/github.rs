use hmac::{Hmac, KeyInit, Mac};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
pub struct WorkflowRunPayload {
    pub action: String,
    pub workflow_run: WorkflowRun,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRun {
    pub name: String,
    pub conclusion: Option<String>,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
}

pub fn build_context(payload: &WorkflowRunPayload) -> serde_json::Value {
    json!({
        "action": payload.action,
        "workflow_run": {
            "name": payload.workflow_run.name,
            "conclusion": payload.workflow_run.conclusion,
            "html_url": payload.workflow_run.html_url,
        },
        "repository": {
            "name": payload.repository.name,
            "full_name": payload.repository.full_name,
        },
    })
}

pub fn verify_signature(secret: &[u8], body: &[u8], signature_header: &str) -> bool {
    let Some(expected_hex) = signature_header.strip_prefix("sha256=") else {
        return false;
    };
    let Some(expected) = decode_hex(expected_hex) else {
        return false;
    };

    let Ok(mut mac) = HmacSha256::new_from_slice(secret) else {
        return false;
    };
    mac.update(body);
    mac.verify_slice(&expected).is_ok()
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }

    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload() -> WorkflowRunPayload {
        WorkflowRunPayload {
            action: "completed".into(),
            workflow_run: WorkflowRun {
                name: "CI".into(),
                conclusion: Some("failure".into()),
                html_url: "https://github.com/my-org/my-repo/actions/runs/1".into(),
            },
            repository: Repository {
                name: "my-repo".into(),
                full_name: "my-org/my-repo".into(),
            },
        }
    }

    fn sign_hex(secret: &[u8], body: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(body);
        let hex: String = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        format!("sha256={hex}")
    }

    #[test]
    fn build_context_exposes_dot_addressable_fields() {
        let context = build_context(&payload());

        assert_eq!(context["workflow_run"]["conclusion"], "failure");
        assert_eq!(context["repository"]["full_name"], "my-org/my-repo");
    }

    #[test]
    fn verify_signature_accepts_a_correctly_signed_body() {
        let body = b"hello world";
        let signature = sign_hex(b"my-secret", body);

        assert!(verify_signature(b"my-secret", body, &signature));
    }

    #[test]
    fn verify_signature_rejects_a_tampered_body() {
        let signature = sign_hex(b"my-secret", b"hello world");

        assert!(!verify_signature(
            b"my-secret",
            b"tampered body",
            &signature
        ));
    }

    #[test]
    fn verify_signature_rejects_the_wrong_secret() {
        let body = b"hello world";
        let signature = sign_hex(b"my-secret", body);

        assert!(!verify_signature(b"wrong-secret", body, &signature));
    }

    #[test]
    fn verify_signature_rejects_missing_prefix() {
        assert!(!verify_signature(b"my-secret", b"hello world", "invalid-signature"));
    }

    #[test]
    fn verify_signature_rejects_invalid_hex() {
        assert!(!verify_signature(
            b"my-secret",
            b"hello world",
            "sha256=not-hex"
        ));
    }
}
