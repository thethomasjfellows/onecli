//! OVHcloud API request signing for the gateway.
//!
//! OVHcloud signs each API request with an application secret, consumer key,
//! method, complete URL, body, and an OVHcloud-supplied timestamp. The three
//! credentials arrive only in internal gateway headers and are removed before
//! the request leaves OneCLI.

use anyhow::{anyhow, Context as _};
use hyper::header::{HeaderName, HeaderValue};
use sha1::{Digest, Sha1};

use super::super::body::buffer_body;

const OVH_APPLICATION_KEY_HEADER: &str = "x-onecli-ovh-application-key";
const OVH_APPLICATION_SECRET_HEADER: &str = "x-onecli-ovh-application-secret";
const OVH_CONSUMER_KEY_HEADER: &str = "x-onecli-ovh-consumer-key";
const OVH_US_API_HOST: &str = "api.us.ovhcloud.com";

#[derive(Clone)]
pub(crate) struct OvhCredentials {
    pub application_key: String,
    pub application_secret: String,
    pub consumer_key: String,
}

/// Extract and remove OVHcloud credentials from internal headers.
///
/// The internal headers are always removed, even when the connection is
/// malformed, so they can never be forwarded upstream.
pub(crate) fn extract_credentials(headers: &mut hyper::HeaderMap) -> Option<OvhCredentials> {
    let application_key = headers.remove(OVH_APPLICATION_KEY_HEADER);
    let application_secret = headers.remove(OVH_APPLICATION_SECRET_HEADER);
    let consumer_key = headers.remove(OVH_CONSUMER_KEY_HEADER);

    let application_key = application_key?.to_str().ok()?.to_owned();
    let application_secret = application_secret?.to_str().ok()?.to_owned();
    let consumer_key = consumer_key?.to_str().ok()?.to_owned();

    Some(OvhCredentials {
        application_key,
        application_secret,
        consumer_key,
    })
}

async fn fetch_timestamp(host: &str) -> anyhow::Result<i64> {
    let response = reqwest::get(format!("https://{host}/1.0/auth/time"))
        .await
        .context("requesting OVHcloud API time")?
        .error_for_status()
        .context("OVHcloud API time request failed")?;
    let text = response
        .text()
        .await
        .context("reading OVHcloud API time response")?;

    text.trim()
        .parse::<i64>()
        .context("parsing OVHcloud API time response")
}

pub(crate) fn sign_request(
    method: &str,
    url: &str,
    headers: &mut hyper::HeaderMap,
    body: &[u8],
    timestamp: i64,
    creds: &OvhCredentials,
) -> anyhow::Result<()> {
    let body = std::str::from_utf8(body)
        .map_err(|_| anyhow!("OVHcloud API request body must be valid UTF-8"))?;

    headers.remove("authorization");
    headers.remove("x-ovh-application");
    headers.remove("x-ovh-consumer");
    headers.remove("x-ovh-signature");
    headers.remove("x-ovh-timestamp");

    let payload = format!(
        "{}+{}+{}+{}+{}+{}",
        creds.application_secret, creds.consumer_key, method, url, body, timestamp
    );
    let signature = format!("$1${:x}", Sha1::digest(payload.as_bytes()));

    headers.insert(
        HeaderName::from_static("x-ovh-application"),
        HeaderValue::from_str(&creds.application_key).context("invalid OVH application key")?,
    );
    headers.insert(
        HeaderName::from_static("x-ovh-consumer"),
        HeaderValue::from_str(&creds.consumer_key).context("invalid OVH consumer key")?,
    );
    headers.insert(
        HeaderName::from_static("x-ovh-signature"),
        HeaderValue::from_str(&signature).context("invalid OVH signature")?,
    );
    headers.insert(
        HeaderName::from_static("x-ovh-timestamp"),
        HeaderValue::from_str(&timestamp.to_string()).context("invalid OVH timestamp")?,
    );

    Ok(())
}

/// Sign an outgoing OVHcloud U.S. request.
pub(crate) async fn finalize_request(
    host: &str,
    method: &str,
    path: &str,
    headers: &mut hyper::HeaderMap,
    body: reqwest::Body,
) -> anyhow::Result<reqwest::Body> {
    let creds = match extract_credentials(headers) {
        Some(credentials) => credentials,
        None => return Ok(body),
    };

    let hostname = host.split(':').next().unwrap_or(host);
    if hostname != OVH_US_API_HOST {
        return Err(anyhow!("OVHcloud signing is only available for {OVH_US_API_HOST}"));
    }

    let body_bytes = buffer_body(body).await?;
    let timestamp = fetch_timestamp(hostname).await?;
    let url = format!("https://{host}{path}");
    sign_request(method, &url, headers, &body_bytes, timestamp, &creds)?;

    tracing::info!(method = %method, host = %host, path = %path, "OVHcloud request signed");

    Ok(reqwest::Body::from(body_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credentials() -> OvhCredentials {
        OvhCredentials {
            application_key: "application-key".into(),
            application_secret: "application-secret".into(),
            consumer_key: "consumer-key".into(),
        }
    }

    #[test]
    fn extract_credentials_removes_only_internal_headers() {
        let mut headers = hyper::HeaderMap::new();
        headers.insert(OVH_APPLICATION_KEY_HEADER, HeaderValue::from_static("application-key"));
        headers.insert(
            OVH_APPLICATION_SECRET_HEADER,
            HeaderValue::from_static("application-secret"),
        );
        headers.insert(OVH_CONSUMER_KEY_HEADER, HeaderValue::from_static("consumer-key"));
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        let creds = extract_credentials(&mut headers).expect("credentials should extract");

        assert_eq!(creds.application_key, "application-key");
        assert_eq!(creds.application_secret, "application-secret");
        assert_eq!(creds.consumer_key, "consumer-key");
        assert!(!headers.contains_key(OVH_APPLICATION_KEY_HEADER));
        assert!(!headers.contains_key(OVH_APPLICATION_SECRET_HEADER));
        assert!(!headers.contains_key(OVH_CONSUMER_KEY_HEADER));
        assert!(headers.contains_key("content-type"));
    }

    #[test]
    fn incomplete_credentials_are_not_forwarded() {
        let mut headers = hyper::HeaderMap::new();
        headers.insert(OVH_APPLICATION_KEY_HEADER, HeaderValue::from_static("application-key"));
        headers.insert(OVH_CONSUMER_KEY_HEADER, HeaderValue::from_static("consumer-key"));

        assert!(extract_credentials(&mut headers).is_none());
        assert!(!headers.contains_key(OVH_APPLICATION_KEY_HEADER));
        assert!(!headers.contains_key(OVH_APPLICATION_SECRET_HEADER));
        assert!(!headers.contains_key(OVH_CONSUMER_KEY_HEADER));
    }

    #[test]
    fn signing_replaces_stale_auth_headers() {
        let mut headers = hyper::HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer stale"));
        headers.insert("x-ovh-signature", HeaderValue::from_static("$1$stale"));

        sign_request(
            "POST",
            "https://api.us.ovhcloud.com/1.0/vps/example",
            &mut headers,
            br#"{"status":"reboot"}"#,
            1_700_000_000,
            &credentials(),
        )
        .expect("request should sign");

        assert!(!headers.contains_key("authorization"));
        assert_eq!(headers["x-ovh-application"], "application-key");
        assert_eq!(headers["x-ovh-consumer"], "consumer-key");
        assert_eq!(headers["x-ovh-timestamp"], "1700000000");
        assert_eq!(
            headers["x-ovh-signature"],
            "$1$ecfad05202f9e11e2e0e6bb38cd7c59c299176df"
        );
    }

    #[test]
    fn signing_rejects_non_utf8_bodies() {
        let error = sign_request(
            "POST",
            "https://api.us.ovhcloud.com/1.0/vps/example",
            &mut hyper::HeaderMap::new(),
            &[0xff],
            1_700_000_000,
            &credentials(),
        )
        .expect_err("binary payloads cannot be signed safely");

        assert!(error.to_string().contains("valid UTF-8"));
    }
}
