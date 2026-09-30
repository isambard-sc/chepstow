use std::time::Duration;

use anyhow::{Context, Result};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{
    AuthType, ClientId, DeviceAuthorizationUrl, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    RefreshToken, RevocationUrl, Scope, StandardDeviceAuthorizationResponse,
    StandardRevocableToken, TokenResponse as _, TokenUrl,
};
use qrcode::{render::unicode, QrCode};
use serde::Deserialize;
use url::Url;

/// The endpoints we need from `{issuer}/.well-known/openid-configuration`
#[derive(Deserialize)]
pub struct Endpoints {
    pub device_authorization_endpoint: DeviceAuthorizationUrl,
    pub token_endpoint: TokenUrl,
    pub revocation_endpoint: Option<RevocationUrl>,
}

pub struct Tokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

type Client =
    BasicClient<EndpointNotSet, EndpointSet, EndpointNotSet, EndpointMaybeSet, EndpointSet>;

pub fn http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("Could not build HTTP client.")
}

pub fn discover(issuer: &str) -> Result<Endpoints> {
    let url = format!(
        "{}/.well-known/openid-configuration",
        issuer.trim_end_matches('/')
    );
    http_client()?
        .get(&url)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.json())
        .with_context(|| format!("OIDC discovery failed at `{url}`."))
}

/// A public client, so the client ID goes in the request body
fn client(ep: &Endpoints, client_id: &str) -> Client {
    BasicClient::new(ClientId::new(client_id.to_string()))
        .set_auth_type(AuthType::RequestBody)
        .set_device_authorization_url(ep.device_authorization_endpoint.clone())
        .set_token_uri(ep.token_endpoint.clone())
        .set_revocation_url_option(ep.revocation_endpoint.clone())
}

fn tokens(r: &BasicTokenResponse) -> Tokens {
    Tokens {
        access_token: r.access_token().secret().clone(),
        refresh_token: r.refresh_token().map(|t| t.secret().clone()),
    }
}

/// Run the device authorization grant. `sleep` is called between polls.
pub fn device_login(
    ep: &Endpoints,
    client_id: &str,
    scope: &str,
    open_browser: bool,
    show_qr: bool,
    sleep: impl Fn(Duration),
) -> Result<Tokens> {
    let http = http_client()?;
    let client = client(ep, client_id);

    let details: StandardDeviceAuthorizationResponse = client
        .exchange_device_code()
        .add_scopes(scope.split_whitespace().map(|s| Scope::new(s.to_string())))
        .request(&http)
        .context("Failed to request codes from device auth endpoint.")?;

    let uri = details
        .verification_uri_complete()
        .context("Did not receive complete verification URI from server.")?
        .secret();
    if open_browser {
        if let Err(e) = webbrowser::open(uri) {
            eprintln!("Could not launch web browser: {e:#}");
        }
    }
    eprintln!("Open this URL in your browser:\n{uri}");
    if show_qr {
        let qr_url = Url::parse_with_params(uri, &[("qr", "1")])?;
        let qr = QrCode::new(qr_url.as_str())?
            .render::<unicode::Dense1x2>()
            .light_color(unicode::Dense1x2::Light)
            .dark_color(unicode::Dense1x2::Dark)
            .build();
        eprintln!("Or scan this QR code:\n{qr}");
    }

    // Handles authorization_pending, slow_down (+5s), expired_token and access_denied
    let token = client
        .exchange_device_access_token(&details)
        .request(&http, sleep, None)
        .context("Could not get token from identity provider.")?;
    Ok(tokens(&token))
}

pub fn refresh(ep: &Endpoints, client_id: &str, refresh_token: &str) -> Result<Tokens> {
    let token = client(ep, client_id)
        .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
        .request(&http_client()?)
        .context("Could not refresh token.")?;
    Ok(tokens(&token))
}

/// Revoke a refresh token. Does nothing if the server has no revocation endpoint.
pub fn revoke(ep: &Endpoints, client_id: &str, refresh_token: &str) -> Result<()> {
    if ep.revocation_endpoint.is_none() {
        return Ok(());
    }
    client(ep, client_id)
        .revoke_token(StandardRevocableToken::RefreshToken(RefreshToken::new(
            refresh_token.to_string(),
        )))?
        .request(&http_client()?)
        .context("Could not revoke token.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn discovery() -> Result<()> {
        let mut server = mockito::Server::new();
        let base = server.url();
        server
            .mock("GET", "/realm/.well-known/openid-configuration")
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"device_authorization_endpoint":"{base}/device","token_endpoint":"{base}/token"}}"#
            ))
            .create();
        let ep = discover(&format!("{base}/realm/"))?;
        assert_eq!(ep.device_authorization_endpoint.as_str(), format!("{base}/device"));
        assert_eq!(ep.token_endpoint.as_str(), format!("{base}/token"));
        assert!(ep.revocation_endpoint.is_none());
        Ok(())
    }

    #[test]
    fn device_flow_pending_slow_down_success() -> Result<()> {
        let mut server = mockito::Server::new();
        let base = server.url();
        server
            .mock("POST", "/device")
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{"device_code":"dc","user_code":"UC","verification_uri":"{base}/v",
                    "verification_uri_complete":"{base}/v?code=UC","expires_in":600,"interval":5}}"#
            ))
            .create();
        let mut token_mock = |status: usize, body: &str| {
            server
                .mock("POST", "/token")
                .with_status(status)
                .with_header("content-type", "application/json")
                .with_body(body)
                .expect(1)
                .create()
        };
        // mockito serves mocks with unmet `expect` in creation order
        let pending = token_mock(400, r#"{"error":"authorization_pending"}"#);
        let slow = token_mock(400, r#"{"error":"slow_down"}"#);
        let ok = token_mock(
            200,
            r#"{"access_token":"at","token_type":"Bearer","refresh_token":"rt","expires_in":300}"#,
        );

        let ep = Endpoints {
            device_authorization_endpoint: DeviceAuthorizationUrl::new(format!("{base}/device"))?,
            token_endpoint: TokenUrl::new(format!("{base}/token"))?,
            revocation_endpoint: None,
        };
        let sleeps = RefCell::new(vec![]);
        let t = device_login(&ep, "chepstow", "openid", false, false, |d| {
            sleeps.borrow_mut().push(d)
        })?;

        assert_eq!(t.access_token, "at");
        assert_eq!(t.refresh_token.as_deref(), Some("rt"));
        assert_eq!(
            sleeps.into_inner(),
            [Duration::from_secs(5), Duration::from_secs(10)]
        );
        pending.assert();
        slow.assert();
        ok.assert();
        Ok(())
    }
}
