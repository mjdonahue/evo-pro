//! Authentication for external services
//!
//! This module provides authentication mechanisms for integrating
//! with external systems and services.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use crate::error::Result;

/// Authentication method types supported by external services
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthMethod {
    /// Basic authentication with username and password
    Basic,

    /// OAuth 2.0 authentication
    OAuth2,

    /// OAuth 2.0 authentication with PKCE (Proof Key for Code Exchange)
    OAuth2Pkce,

    /// API key authentication
    ApiKey,

    /// JWT token authentication
    Jwt,

    /// Multi-factor authentication
    MultiFactorAuth,

    /// Custom authentication method
    Custom(String),
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthMethod::Basic => write!(f, "Basic"),
            AuthMethod::OAuth2 => write!(f, "OAuth2"),
            AuthMethod::OAuth2Pkce => write!(f, "OAuth2PKCE"),
            AuthMethod::ApiKey => write!(f, "ApiKey"),
            AuthMethod::Jwt => write!(f, "JWT"),
            AuthMethod::MultiFactorAuth => write!(f, "MultiFactorAuth"),
            AuthMethod::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Authentication credentials for external services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Unique identifier for these credentials
    pub id: String,

    /// Service these credentials are for
    pub service_id: String,

    /// Authentication method
    pub method: AuthMethod,

    /// Credential parameters (e.g., username, password, token)
    #[serde(skip_serializing)]
    pub parameters: HashMap<String, String>,

    /// When these credentials were created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When these credentials were last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// When these credentials expire (if applicable)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Authentication token for external services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// The actual token value
    pub token: String,

    /// Type of the token (e.g., "Bearer", "Basic")
    pub token_type: String,

    /// When the token expires
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Refresh token (if applicable)
    pub refresh_token: Option<String>,

    /// Scope of the token
    pub scope: Option<String>,

    /// Additional token properties
    pub properties: HashMap<String, String>,
}

impl AuthToken {
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expiry) => chrono::Utc::now() >= expiry,
            None => false,
        }
    }

    /// Get the remaining validity duration of the token
    pub fn validity_duration(&self) -> Option<Duration> {
        self.expires_at.map(|expiry| {
            let now = chrono::Utc::now();
            if now >= expiry {
                Duration::from_secs(0)
            } else {
                let diff = expiry.signed_duration_since(now);
                Duration::from_secs(diff.num_seconds().max(0) as u64)
            }
        })
    }
}

/// OAuth2 configuration for external services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// Client ID
    pub client_id: String,

    /// Client secret
    #[serde(skip_serializing)]
    pub client_secret: String,

    /// Authorization endpoint URL
    pub auth_url: String,

    /// Token endpoint URL
    pub token_url: String,

    /// Redirect URL for the OAuth flow
    pub redirect_url: String,

    /// Scopes to request
    pub scopes: Vec<String>,

    /// Additional parameters for the OAuth flow
    pub additional_params: HashMap<String, String>,
}

/// OAuth2 with PKCE configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2PkceConfig {
    /// Base OAuth2 configuration
    pub oauth2_config: OAuth2Config,

    /// Code challenge method (S256 or plain)
    pub code_challenge_method: String,

    /// Whether to use state parameter for CSRF protection
    pub use_state: bool,
}

/// Trait for authentication providers
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get the authentication method supported by this provider
    fn auth_method(&self) -> AuthMethod;

    /// Authenticate with the external service
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken>;

    /// Refresh an authentication token
    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken>;

    /// Revoke an authentication token
    async fn revoke_token(&self, token: &AuthToken) -> Result<()>;

    /// Check if a token is valid
    async fn validate_token(&self, token: &AuthToken) -> Result<bool>;
}

/// Basic authentication provider
pub struct BasicAuthProvider;

#[async_trait]
impl AuthProvider for BasicAuthProvider {
    fn auth_method(&self) -> AuthMethod {
        AuthMethod::Basic
    }

    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // Implementation would create a Basic auth token from username/password
        // This is a placeholder implementation
        let username = credentials.parameters.get("username").ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "Username not provided for Basic authentication"
            )
        })?;

        let password = credentials.parameters.get("password").ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "Password not provided for Basic authentication"
            )
        })?;

        // In a real implementation, we would properly encode the credentials
        // and possibly make a test request to validate them
        let token_value = format!("{}:{}", username, password);
        let encoded = base64::encode(token_value);

        Ok(AuthToken {
            token: encoded,
            token_type: "Basic".to_string(),
            expires_at: None, // Basic auth doesn't expire
            refresh_token: None,
            scope: None,
            properties: HashMap::new(),
        })
    }

    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken> {
        // Basic auth doesn't support token refresh
        Ok(token.clone())
    }

    async fn revoke_token(&self, _token: &AuthToken) -> Result<()> {
        // Basic auth doesn't support token revocation
        Ok(())
    }

    async fn validate_token(&self, _token: &AuthToken) -> Result<bool> {
        // Basic auth tokens don't expire, so they're always valid
        // In a real implementation, we might make a test request to validate
        Ok(true)
    }
}

/// OAuth2 authentication provider
pub struct OAuth2Provider {
    config: OAuth2Config,
}

impl OAuth2Provider {
    /// Create a new OAuth2 provider with the given configuration
    pub fn new(config: OAuth2Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AuthProvider for OAuth2Provider {
    fn auth_method(&self) -> AuthMethod {
        AuthMethod::OAuth2
    }

    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // Implementation would perform OAuth2 flow
        // This is a placeholder implementation
        let auth_code = credentials.parameters.get("auth_code").ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "Authorization code not provided for OAuth2 authentication"
            )
        })?;

        // In a real implementation, we would exchange the auth code for tokens
        // using the OAuth2 token endpoint

        // Simulate token expiration in 1 hour
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        Ok(AuthToken {
            token: format!("simulated_access_token_{}", auth_code),
            token_type: "Bearer".to_string(),
            expires_at: Some(expires_at),
            refresh_token: Some("simulated_refresh_token".to_string()),
            scope: Some(self.config.scopes.join(" ")),
            properties: HashMap::new(),
        })
    }

    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken> {
        // Implementation would refresh the OAuth2 token
        // This is a placeholder implementation
        let refresh_token = token.refresh_token.as_ref().ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "No refresh token available"
            )
        })?;

        // In a real implementation, we would use the refresh token to get a new access token
        // using the OAuth2 token endpoint

        // Simulate token expiration in 1 hour
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        Ok(AuthToken {
            token: format!("refreshed_access_token_{}", refresh_token),
            token_type: "Bearer".to_string(),
            expires_at: Some(expires_at),
            refresh_token: Some(refresh_token.clone()),
            scope: token.scope.clone(),
            properties: token.properties.clone(),
        })
    }

    async fn revoke_token(&self, token: &AuthToken) -> Result<()> {
        // Implementation would revoke the OAuth2 token
        // This is a placeholder implementation
        let _refresh_token = token.refresh_token.as_ref().ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "No refresh token available to revoke"
            )
        })?;

        // In a real implementation, we would call the OAuth2 revocation endpoint

        Ok(())
    }

    async fn validate_token(&self, token: &AuthToken) -> Result<bool> {
        // Check if the token is expired
        if token.is_expired() {
            return Ok(false);
        }

        // In a real implementation, we might make a request to the OAuth2 introspection endpoint

        Ok(true)
    }
}

/// JWT configuration for external services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Issuer of the JWT
    pub issuer: String,

    /// Audience of the JWT
    pub audience: Option<String>,

    /// Secret key for HMAC algorithms
    #[serde(skip_serializing)]
    pub secret_key: Option<String>,

    /// Public key for RSA/ECDSA algorithms
    pub public_key: Option<String>,

    /// Private key for RSA/ECDSA algorithms
    #[serde(skip_serializing)]
    pub private_key: Option<String>,

    /// Algorithm to use for signing/verifying
    pub algorithm: String,

    /// Token lifetime in seconds
    pub token_lifetime_seconds: u64,
}

/// JWT authentication provider
pub struct JwtAuthProvider {
    config: JwtConfig,
}

impl JwtAuthProvider {
    /// Create a new JWT provider with the given configuration
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    /// Generate a JWT token
    fn generate_token(&self, claims: HashMap<String, String>) -> Result<String> {
        // In a real implementation, we would use a JWT library to generate the token
        // This is a placeholder implementation

        // Create a header
        let header = format!("{{\"alg\":\"{}\",\"typ\":\"JWT\"}}", self.config.algorithm);
        let header_base64 = base64::encode(header);

        // Create the claims
        let mut claims_map = claims;
        claims_map.insert("iss".to_string(), self.config.issuer.clone());
        if let Some(aud) = &self.config.audience {
            claims_map.insert("aud".to_string(), aud.clone());
        }

        // Add expiration time
        let exp = chrono::Utc::now() + chrono::Duration::seconds(self.config.token_lifetime_seconds as i64);
        claims_map.insert("exp".to_string(), exp.timestamp().to_string());

        // Add issued at time
        let iat = chrono::Utc::now();
        claims_map.insert("iat".to_string(), iat.timestamp().to_string());

        // Serialize claims to JSON
        let claims_json = serde_json::to_string(&claims_map).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                &format!("Failed to serialize JWT claims: {}", e)
            )
        })?;

        let claims_base64 = base64::encode(claims_json);

        // Create the signature (in a real implementation, this would use the appropriate algorithm)
        let signature = match self.config.algorithm.as_str() {
            "HS256" | "HS384" | "HS512" => {
                let secret = self.config.secret_key.as_ref().ok_or_else(|| {
                    crate::error::Error::new(
                        crate::error::ErrorKind::Configuration,
                        "Secret key required for HMAC algorithms"
                    )
                })?;

                // In a real implementation, we would use a proper HMAC function
                // This is just a placeholder
                base64::encode(format!("hmac_signature_for_{}_{}", header_base64, claims_base64))
            },
            "RS256" | "RS384" | "RS512" | "ES256" | "ES384" | "ES512" => {
                let private_key = self.config.private_key.as_ref().ok_or_else(|| {
                    crate::error::Error::new(
                        crate::error::ErrorKind::Configuration,
                        "Private key required for RSA/ECDSA algorithms"
                    )
                })?;

                // In a real implementation, we would use a proper RSA/ECDSA signing function
                // This is just a placeholder
                base64::encode(format!("rsa_signature_for_{}_{}", header_base64, claims_base64))
            },
            _ => {
                return Err(crate::error::Error::new(
                    crate::error::ErrorKind::Configuration,
                    &format!("Unsupported JWT algorithm: {}", self.config.algorithm)
                ));
            }
        };

        // Combine the parts to form the JWT
        Ok(format!("{}.{}.{}", header_base64, claims_base64, signature))
    }

    /// Verify a JWT token
    fn verify_token(&self, token: &str) -> Result<HashMap<String, String>> {
        // In a real implementation, we would use a JWT library to verify the token
        // This is a placeholder implementation

        // Split the token into parts
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "Invalid JWT format"
            ));
        }

        // Decode the claims
        let claims_json = base64::decode(parts[1]).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                &format!("Failed to decode JWT claims: {}", e)
            )
        })?;

        let claims: HashMap<String, String> = serde_json::from_slice(&claims_json).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                &format!("Failed to parse JWT claims: {}", e)
            )
        })?;

        // Verify the token hasn't expired
        if let Some(exp_str) = claims.get("exp") {
            if let Ok(exp) = exp_str.parse::<i64>() {
                let exp_time = chrono::DateTime::<chrono::Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp_opt(exp, 0).unwrap_or_default(),
                    chrono::Utc,
                );

                if chrono::Utc::now() > exp_time {
                    return Err(crate::error::Error::new(
                        crate::error::ErrorKind::Authentication,
                        "JWT token has expired"
                    ));
                }
            }
        }

        // Verify the issuer
        if let Some(iss) = claims.get("iss") {
            if *iss != self.config.issuer {
                return Err(crate::error::Error::new(
                    crate::error::ErrorKind::Authentication,
                    "JWT issuer does not match"
                ));
            }
        }

        // Verify the audience if specified
        if let Some(expected_aud) = &self.config.audience {
            if let Some(aud) = claims.get("aud") {
                if aud != expected_aud {
                    return Err(crate::error::Error::new(
                        crate::error::ErrorKind::Authentication,
                        "JWT audience does not match"
                    ));
                }
            }
        }

        // In a real implementation, we would verify the signature using the appropriate algorithm

        Ok(claims)
    }
}

#[async_trait]
impl AuthProvider for JwtAuthProvider {
    fn auth_method(&self) -> AuthMethod {
        AuthMethod::Jwt
    }

    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // Extract claims from credentials
        let mut claims = HashMap::new();

        // Add subject if provided
        if let Some(sub) = credentials.parameters.get("sub") {
            claims.insert("sub".to_string(), sub.clone());
        }

        // Add any additional claims
        for (key, value) in &credentials.parameters {
            if key != "sub" && !key.starts_with("_") {
                claims.insert(key.clone(), value.clone());
            }
        }

        // Generate the JWT
        let token = self.generate_token(claims)?;

        // Calculate expiration time
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.config.token_lifetime_seconds as i64);

        Ok(AuthToken {
            token,
            token_type: "Bearer".to_string(),
            expires_at: Some(expires_at),
            refresh_token: None, // JWTs typically don't use refresh tokens
            scope: None,
            properties: HashMap::new(),
        })
    }

    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken> {
        // JWTs are typically not refreshed, but reissued
        // We'll extract the claims from the old token and generate a new one

        let claims = self.verify_token(&token.token)?;

        // Generate a new token with the same claims
        let new_token = self.generate_token(claims)?;

        // Calculate expiration time
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.config.token_lifetime_seconds as i64);

        Ok(AuthToken {
            token: new_token,
            token_type: "Bearer".to_string(),
            expires_at: Some(expires_at),
            refresh_token: None,
            scope: token.scope.clone(),
            properties: token.properties.clone(),
        })
    }

    async fn revoke_token(&self, _token: &AuthToken) -> Result<()> {
        // JWTs can't be revoked directly
        // In a real implementation, we might add the token to a blacklist

        // For now, we'll just return success
        Ok(())
    }

    async fn validate_token(&self, token: &AuthToken) -> Result<bool> {
        // Verify the token
        match self.verify_token(&token.token) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// API key authentication provider
pub struct ApiKeyProvider;

#[async_trait]
impl AuthProvider for ApiKeyProvider {
    fn auth_method(&self) -> AuthMethod {
        AuthMethod::ApiKey
    }

    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // Implementation would validate and return an API key token
        // This is a placeholder implementation
        let api_key = credentials.parameters.get("api_key").ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "API key not provided for API key authentication"
            )
        })?;

        // In a real implementation, we might validate the API key format

        Ok(AuthToken {
            token: api_key.clone(),
            token_type: "ApiKey".to_string(),
            expires_at: None, // API keys typically don't expire
            refresh_token: None,
            scope: None,
            properties: HashMap::new(),
        })
    }

    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken> {
        // API keys don't support token refresh
        Ok(token.clone())
    }

    async fn revoke_token(&self, _token: &AuthToken) -> Result<()> {
        // API keys don't support token revocation in the traditional sense
        // In a real implementation, we might invalidate the API key in a database
        Ok(())
    }

    async fn validate_token(&self, _token: &AuthToken) -> Result<bool> {
        // API keys don't expire, so they're always valid
        // In a real implementation, we might check if the API key is still valid in a database
        Ok(true)
    }
}

/// OAuth2 with PKCE authentication provider
pub struct OAuth2PkceProvider {
    config: OAuth2PkceConfig,
}

impl OAuth2PkceProvider {
    /// Create a new OAuth2 with PKCE provider with the given configuration
    pub fn new(config: OAuth2PkceConfig) -> Self {
        Self { config }
    }

    /// Generate a random code verifier
    fn generate_code_verifier() -> String {
        use rand::{thread_rng, Rng};
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

        let mut rng = thread_rng();
        let verifier_length = 64; // Between 43 and 128 characters per spec

        (0..verifier_length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Generate a code challenge from a code verifier
    fn generate_code_challenge(&self, verifier: &str) -> Result<String> {
        match self.config.code_challenge_method.as_str() {
            "plain" => {
                // For plain method, the challenge is the same as the verifier
                Ok(verifier.to_string())
            },
            "S256" => {
                // For S256 method, the challenge is the base64url-encoded SHA-256 hash of the verifier
                use sha2::{Sha256, Digest};

                let mut hasher = Sha256::new();
                hasher.update(verifier.as_bytes());
                let hash = hasher.finalize();

                // Base64url encode the hash
                // Note: This is a simplified implementation
                let encoded = base64::encode(&hash);
                let encoded = encoded.replace('+', "-").replace('/', "_").replace('=', "");

                Ok(encoded)
            },
            _ => {
                Err(crate::error::Error::new(
                    crate::error::ErrorKind::Configuration,
                    &format!("Unsupported code challenge method: {}", self.config.code_challenge_method)
                ))
            }
        }
    }

    /// Generate a random state parameter for CSRF protection
    fn generate_state() -> String {
        use rand::{thread_rng, Rng};
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

        let mut rng = thread_rng();
        let state_length = 32;

        (0..state_length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Generate the authorization URL with PKCE parameters
    pub fn generate_authorization_url(&self) -> Result<(String, String, Option<String>)> {
        // Generate code verifier
        let code_verifier = Self::generate_code_verifier();

        // Generate code challenge
        let code_challenge = self.generate_code_challenge(&code_verifier)?;

        // Generate state if enabled
        let state = if self.config.use_state {
            Some(Self::generate_state())
        } else {
            None
        };

        // Build the authorization URL
        let mut url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&code_challenge={}&code_challenge_method={}",
            self.config.oauth2_config.auth_url,
            urlencoding::encode(&self.config.oauth2_config.client_id),
            urlencoding::encode(&self.config.oauth2_config.redirect_url),
            urlencoding::encode(&code_challenge),
            urlencoding::encode(&self.config.code_challenge_method)
        );

        // Add scopes
        if !self.config.oauth2_config.scopes.is_empty() {
            url.push_str("&scope=");
            url.push_str(&urlencoding::encode(&self.config.oauth2_config.scopes.join(" ")));
        }

        // Add state if present
        if let Some(state_value) = &state {
            url.push_str("&state=");
            url.push_str(&urlencoding::encode(state_value));
        }

        // Add additional parameters
        for (key, value) in &self.config.oauth2_config.additional_params {
            url.push_str(&format!("&{}={}", urlencoding::encode(key), urlencoding::encode(value)));
        }

        Ok((url, code_verifier, state))
    }

    /// Exchange authorization code for tokens using PKCE
    async fn exchange_code_for_tokens(&self, auth_code: &str, code_verifier: &str) -> Result<AuthToken> {
        // In a real implementation, we would make an HTTP request to the token endpoint
        // This is a placeholder implementation

        // Build the request body
        let mut body = format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
            urlencoding::encode(auth_code),
            urlencoding::encode(&self.config.oauth2_config.redirect_url),
            urlencoding::encode(&self.config.oauth2_config.client_id),
            urlencoding::encode(code_verifier)
        );

        // Add client secret if present
        if !self.config.oauth2_config.client_secret.is_empty() {
            body.push_str(&format!("&client_secret={}", 
                urlencoding::encode(&self.config.oauth2_config.client_secret)));
        }

        // In a real implementation, we would send this request to the token endpoint
        // and parse the response to extract the tokens

        // Simulate token expiration in 1 hour
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        Ok(AuthToken {
            token: format!("simulated_access_token_with_pkce_{}", auth_code),
            token_type: "Bearer".to_string(),
            expires_at: Some(expires_at),
            refresh_token: Some("simulated_refresh_token_with_pkce".to_string()),
            scope: Some(self.config.oauth2_config.scopes.join(" ")),
            properties: HashMap::new(),
        })
    }
}

#[async_trait]
impl AuthProvider for OAuth2PkceProvider {
    fn auth_method(&self) -> AuthMethod {
        AuthMethod::OAuth2Pkce
    }

    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // Get the authorization code and code verifier from credentials
        let auth_code = credentials.parameters.get("auth_code").ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "Authorization code not provided for OAuth2 PKCE authentication"
            )
        })?;

        let code_verifier = credentials.parameters.get("code_verifier").ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "Code verifier not provided for OAuth2 PKCE authentication"
            )
        })?;

        // Verify state if enabled
        if self.config.use_state {
            let expected_state = credentials.parameters.get("expected_state").ok_or_else(|| {
                crate::error::Error::new(
                    crate::error::ErrorKind::Authentication,
                    "Expected state not provided for OAuth2 PKCE authentication"
                )
            })?;

            let actual_state = credentials.parameters.get("actual_state").ok_or_else(|| {
                crate::error::Error::new(
                    crate::error::ErrorKind::Authentication,
                    "Actual state not provided for OAuth2 PKCE authentication"
                )
            })?;

            if expected_state != actual_state {
                return Err(crate::error::Error::new(
                    crate::error::ErrorKind::Authentication,
                    "State mismatch in OAuth2 PKCE flow"
                ));
            }
        }

        // Exchange the authorization code for tokens
        self.exchange_code_for_tokens(auth_code, code_verifier).await
    }

    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken> {
        // Get the refresh token
        let refresh_token = token.refresh_token.as_ref().ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "No refresh token available"
            )
        })?;

        // In a real implementation, we would make an HTTP request to the token endpoint
        // with the refresh token to get a new access token

        // Simulate token expiration in 1 hour
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        Ok(AuthToken {
            token: format!("refreshed_access_token_with_pkce_{}", refresh_token),
            token_type: "Bearer".to_string(),
            expires_at: Some(expires_at),
            refresh_token: Some(refresh_token.clone()),
            scope: token.scope.clone(),
            properties: token.properties.clone(),
        })
    }

    async fn revoke_token(&self, token: &AuthToken) -> Result<()> {
        // Get the refresh token
        let _refresh_token = token.refresh_token.as_ref().ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "No refresh token available to revoke"
            )
        })?;

        // In a real implementation, we would make an HTTP request to the revocation endpoint

        Ok(())
    }

    async fn validate_token(&self, token: &AuthToken) -> Result<bool> {
        // Check if the token is expired
        if token.is_expired() {
            return Ok(false);
        }

        // In a real implementation, we might make a request to the introspection endpoint

        Ok(true)
    }
}

/// Factory for creating authentication providers
pub struct AuthProviderFactory {
    providers: HashMap<AuthMethod, Box<dyn AuthProvider>>,
}

impl AuthProviderFactory {
    /// Create a new authentication provider factory
    pub fn new() -> Self {
        let mut providers = HashMap::new();
        providers.insert(AuthMethod::Basic, Box::new(BasicAuthProvider) as Box<dyn AuthProvider>);
        providers.insert(AuthMethod::ApiKey, Box::new(ApiKeyProvider) as Box<dyn AuthProvider>);
        // OAuth2 provider requires configuration, so it's not added by default

        Self { providers }
    }

    /// Register a custom authentication provider
    pub fn register_provider(&mut self, provider: Box<dyn AuthProvider>) {
        let method = provider.auth_method();
        self.providers.insert(method, provider);
    }

    /// Get an authentication provider for the given method
    pub fn get_provider(&self, method: &AuthMethod) -> Option<&dyn AuthProvider> {
        self.providers.get(method).map(|p| p.as_ref())
    }

    /// Get the list of supported authentication methods
    pub fn supported_methods(&self) -> Vec<AuthMethod> {
        self.providers.keys().cloned().collect()
    }
}

/// Credential store for securely storing and retrieving credentials
pub trait CredentialStore: Send + Sync {
    /// Store credentials
    fn store_credentials(&self, credentials: &Credentials) -> Result<()>;

    /// Retrieve credentials by ID
    fn get_credentials(&self, id: &str) -> Result<Credentials>;

    /// Retrieve credentials for a specific service
    fn get_credentials_for_service(&self, service_id: &str) -> Result<Vec<Credentials>>;

    /// Delete credentials
    fn delete_credentials(&self, id: &str) -> Result<()>;

    /// Update credentials
    fn update_credentials(&self, credentials: &Credentials) -> Result<()>;
}

/// In-memory credential store (for testing purposes)
pub struct InMemoryCredentialStore {
    credentials: std::sync::RwLock<HashMap<String, Credentials>>,
}

impl InMemoryCredentialStore {
    /// Create a new in-memory credential store
    pub fn new() -> Self {
        Self {
            credentials: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn store_credentials(&self, credentials: &Credentials) -> Result<()> {
        let mut store = self.credentials.write().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire write lock on credential store"
            )
        })?;

        store.insert(credentials.id.clone(), credentials.clone());
        Ok(())
    }

    fn get_credentials(&self, id: &str) -> Result<Credentials> {
        let store = self.credentials.read().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire read lock on credential store"
            )
        })?;

        store.get(id).cloned().ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::NotFound,
                &format!("Credentials with ID {} not found", id)
            )
        })
    }

    fn get_credentials_for_service(&self, service_id: &str) -> Result<Vec<Credentials>> {
        let store = self.credentials.read().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire read lock on credential store"
            )
        })?;

        let creds = store.values()
            .filter(|c| c.service_id == service_id)
            .cloned()
            .collect();

        Ok(creds)
    }

    fn delete_credentials(&self, id: &str) -> Result<()> {
        let mut store = self.credentials.write().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire write lock on credential store"
            )
        })?;

        store.remove(id).ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::NotFound,
                &format!("Credentials with ID {} not found", id)
            )
        })?;

        Ok(())
    }

    fn update_credentials(&self, credentials: &Credentials) -> Result<()> {
        let mut store = self.credentials.write().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire write lock on credential store"
            )
        })?;

        if !store.contains_key(&credentials.id) {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::NotFound,
                &format!("Credentials with ID {} not found", credentials.id)
            ));
        }

        store.insert(credentials.id.clone(), credentials.clone());
        Ok(())
    }
}

/// Configuration for the encrypted file credential store
#[derive(Debug, Clone)]
pub struct EncryptedFileCredentialStoreConfig {
    /// Path to the credential store file
    pub file_path: std::path::PathBuf,

    /// Encryption key derivation method
    pub key_derivation: KeyDerivationMethod,
}

/// Key derivation methods for credential encryption
#[derive(Debug, Clone)]
pub enum KeyDerivationMethod {
    /// Derive key from password using PBKDF2
    Password {
        /// Password for encryption/decryption
        password: String,

        /// Salt for key derivation
        salt: Vec<u8>,

        /// Number of iterations for key derivation
        iterations: u32,
    },

    /// Use a system-protected key (e.g., keychain, keyring)
    SystemProtected {
        /// Key identifier in the system keystore
        key_id: String,
    },

    /// Use a hardware security module
    Hsm {
        /// Key identifier in the HSM
        key_id: String,

        /// HSM configuration
        config: HashMap<String, String>,
    },
}

/// Encrypted file-based credential store
pub struct EncryptedFileCredentialStore {
    /// Configuration for the credential store
    config: EncryptedFileCredentialStoreConfig,

    /// Cache of decrypted credentials
    cache: std::sync::RwLock<HashMap<String, Credentials>>,

    /// Whether the store has been initialized
    initialized: std::sync::atomic::AtomicBool,
}

impl EncryptedFileCredentialStore {
    /// Create a new encrypted file credential store
    pub fn new(config: EncryptedFileCredentialStoreConfig) -> Self {
        Self {
            config,
            cache: std::sync::RwLock::new(HashMap::new()),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Initialize the credential store
    pub fn initialize(&self) -> Result<()> {
        // Check if the store is already initialized
        if self.initialized.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }

        // Create the directory if it doesn't exist
        if let Some(parent) = self.config.file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::error::Error::new(
                    crate::error::ErrorKind::IO,
                    &format!("Failed to create directory for credential store: {}", e)
                )
            })?;
        }

        // If the file doesn't exist, create it with an empty set of credentials
        if !self.config.file_path.exists() {
            self.save_to_file(&HashMap::new())?;
        }

        // Load credentials from the file
        self.load_from_file()?;

        // Mark as initialized
        self.initialized.store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    /// Derive encryption key from the configuration
    fn derive_key(&self) -> Result<Vec<u8>> {
        match &self.config.key_derivation {
            KeyDerivationMethod::Password { password, salt, iterations } => {
                use ring::pbkdf2;

                let mut key = [0u8; 32]; // 256-bit key
                pbkdf2::derive(
                    pbkdf2::PBKDF2_HMAC_SHA256,
                    std::num::NonZeroU32::new(*iterations).unwrap(),
                    salt,
                    password.as_bytes(),
                    &mut key,
                );

                Ok(key.to_vec())
            },
            KeyDerivationMethod::SystemProtected { key_id } => {
                // In a real implementation, we would use the system keystore
                // This is a placeholder implementation

                // Generate a deterministic key from the key ID
                let mut hasher = sha2::Sha256::new();
                hasher.update(key_id.as_bytes());
                let hash = hasher.finalize();

                Ok(hash.to_vec())
            },
            KeyDerivationMethod::Hsm { key_id, config: _ } => {
                // In a real implementation, we would use the HSM
                // This is a placeholder implementation

                // Generate a deterministic key from the key ID
                let mut hasher = sha2::Sha256::new();
                hasher.update(key_id.as_bytes());
                let hash = hasher.finalize();

                Ok(hash.to_vec())
            },
        }
    }

    /// Encrypt data using the derived key
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, Key, Nonce};
        use aes_gcm::aead::{Aead, NewAead};
        use rand::{thread_rng, Rng};

        // Derive the encryption key
        let key_bytes = self.derive_key()?;
        let key = Key::from_slice(&key_bytes);

        // Create the cipher
        let cipher = Aes256Gcm::new(key);

        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher.encrypt(nonce, data).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::Security,
                &format!("Failed to encrypt data: {}", e)
            )
        })?;

        // Combine nonce and ciphertext
        let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using the derived key
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, Key, Nonce};
        use aes_gcm::aead::{Aead, NewAead};

        // Derive the encryption key
        let key_bytes = self.derive_key()?;
        let key = Key::from_slice(&key_bytes);

        // Create the cipher
        let cipher = Aes256Gcm::new(key);

        // Split the data into nonce and ciphertext
        if data.len() < 12 {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::Security,
                "Invalid encrypted data format"
            ));
        }

        let nonce = Nonce::from_slice(&data[0..12]);
        let ciphertext = &data[12..];

        // Decrypt the data
        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::Security,
                &format!("Failed to decrypt data: {}", e)
            )
        })?;

        Ok(plaintext)
    }

    /// Load credentials from the encrypted file
    fn load_from_file(&self) -> Result<()> {
        // Read the encrypted data from the file
        let encrypted_data = std::fs::read(&self.config.file_path).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::IO,
                &format!("Failed to read credential store file: {}", e)
            )
        })?;

        // If the file is empty, return an empty set of credentials
        if encrypted_data.is_empty() {
            let mut cache = self.cache.write().map_err(|_| {
                crate::error::Error::new(
                    crate::error::ErrorKind::Internal,
                    "Failed to acquire write lock on credential store cache"
                )
            })?;

            cache.clear();
            return Ok(());
        }

        // Decrypt the data
        let decrypted_data = self.decrypt(&encrypted_data)?;

        // Deserialize the credentials
        let credentials: HashMap<String, Credentials> = serde_json::from_slice(&decrypted_data).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::Parse,
                &format!("Failed to deserialize credentials: {}", e)
            )
        })?;

        // Update the cache
        let mut cache = self.cache.write().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire write lock on credential store cache"
            )
        })?;

        *cache = credentials;

        Ok(())
    }

    /// Save credentials to the encrypted file
    fn save_to_file(&self, credentials: &HashMap<String, Credentials>) -> Result<()> {
        // Serialize the credentials
        let data = serde_json::to_vec(credentials).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                &format!("Failed to serialize credentials: {}", e)
            )
        })?;

        // Encrypt the data
        let encrypted_data = self.encrypt(&data)?;

        // Write the encrypted data to the file
        std::fs::write(&self.config.file_path, &encrypted_data).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::IO,
                &format!("Failed to write credential store file: {}", e)
            )
        })?;

        Ok(())
    }
}

impl CredentialStore for EncryptedFileCredentialStore {
    fn store_credentials(&self, credentials: &Credentials) -> Result<()> {
        // Initialize if needed
        self.initialize()?;

        // Update the cache
        let mut cache = self.cache.write().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire write lock on credential store cache"
            )
        })?;

        cache.insert(credentials.id.clone(), credentials.clone());

        // Save to file
        self.save_to_file(&cache)?;

        Ok(())
    }

    fn get_credentials(&self, id: &str) -> Result<Credentials> {
        // Initialize if needed
        self.initialize()?;

        // Get from cache
        let cache = self.cache.read().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire read lock on credential store cache"
            )
        })?;

        cache.get(id).cloned().ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::NotFound,
                &format!("Credentials with ID {} not found", id)
            )
        })
    }

    fn get_credentials_for_service(&self, service_id: &str) -> Result<Vec<Credentials>> {
        // Initialize if needed
        self.initialize()?;

        // Get from cache
        let cache = self.cache.read().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire read lock on credential store cache"
            )
        })?;

        let creds = cache.values()
            .filter(|c| c.service_id == service_id)
            .cloned()
            .collect();

        Ok(creds)
    }

    fn delete_credentials(&self, id: &str) -> Result<()> {
        // Initialize if needed
        self.initialize()?;

        // Update the cache
        let mut cache = self.cache.write().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire write lock on credential store cache"
            )
        })?;

        if !cache.contains_key(id) {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::NotFound,
                &format!("Credentials with ID {} not found", id)
            ));
        }

        cache.remove(id);

        // Save to file
        self.save_to_file(&cache)?;

        Ok(())
    }

    fn update_credentials(&self, credentials: &Credentials) -> Result<()> {
        // Initialize if needed
        self.initialize()?;

        // Update the cache
        let mut cache = self.cache.write().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire write lock on credential store cache"
            )
        })?;

        if !cache.contains_key(&credentials.id) {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::NotFound,
                &format!("Credentials with ID {} not found", credentials.id)
            ));
        }

        cache.insert(credentials.id.clone(), credentials.clone());

        // Save to file
        self.save_to_file(&cache)?;

        Ok(())
    }
}

/// Multi-factor authentication factor types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MfaFactorType {
    /// Time-based one-time password (TOTP)
    Totp,

    /// HMAC-based one-time password (HOTP)
    Hotp,

    /// SMS-based one-time password
    Sms,

    /// Email-based one-time password
    Email,

    /// Push notification
    Push,

    /// Security key (e.g., FIDO2, WebAuthn)
    SecurityKey,

    /// Backup codes
    BackupCodes,

    /// Custom factor type
    Custom(String),
}

/// Multi-factor authentication factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaFactor {
    /// Unique identifier for this factor
    pub id: String,

    /// Type of factor
    pub factor_type: MfaFactorType,

    /// Name of the factor (e.g., "Work Phone", "Personal Email")
    pub name: String,

    /// Whether this factor is enabled
    pub enabled: bool,

    /// Last time this factor was used
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,

    /// Factor-specific configuration
    pub config: HashMap<String, String>,
}

/// Multi-factor authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiFactorAuthConfig {
    /// Enabled factor types
    pub enabled_factors: Vec<MfaFactorType>,

    /// Number of factors required for authentication
    pub required_factors: u32,

    /// Whether to allow remembering devices
    pub allow_remember_device: bool,

    /// How long to remember devices (in seconds)
    pub remember_device_seconds: Option<u64>,

    /// Whether to allow backup codes
    pub allow_backup_codes: bool,

    /// Number of backup codes to generate
    pub backup_codes_count: Option<u32>,
}

/// Multi-factor authentication provider
pub struct MultiFactorAuthProvider {
    config: MultiFactorAuthConfig,
    base_provider: Box<dyn AuthProvider>,
}

impl MultiFactorAuthProvider {
    /// Create a new multi-factor authentication provider
    pub fn new(config: MultiFactorAuthConfig, base_provider: Box<dyn AuthProvider>) -> Self {
        Self { config, base_provider }
    }

    /// Generate a TOTP code
    fn generate_totp(&self, secret: &str, time_step: u64) -> Result<String> {
        // In a real implementation, we would use a TOTP library
        // This is a placeholder implementation

        // Get the current time
        let current_time = chrono::Utc::now().timestamp() as u64;

        // Calculate the time counter
        let counter = current_time / time_step;

        // Generate a deterministic code based on the secret and counter
        let mut hasher = sha2::Sha256::new();
        hasher.update(secret.as_bytes());
        hasher.update(&counter.to_be_bytes());
        let hash = hasher.finalize();

        // Take the first 6 digits
        let code = format!("{:x}", hash)[0..6].to_string();

        Ok(code)
    }

    /// Verify a TOTP code
    fn verify_totp(&self, secret: &str, code: &str, time_step: u64) -> Result<bool> {
        // Generate the current code
        let current_code = self.generate_totp(secret, time_step)?;

        // Check if the codes match
        Ok(current_code == code)
    }

    /// Generate backup codes
    fn generate_backup_codes(&self, count: u32) -> Vec<String> {
        use rand::{thread_rng, Rng};

        let mut rng = thread_rng();
        let mut codes = Vec::with_capacity(count as usize);

        for _ in 0..count {
            // Generate a random 8-digit code
            let code: u32 = rng.gen_range(10000000..99999999);
            codes.push(format!("{}", code));
        }

        codes
    }

    /// Verify a backup code
    fn verify_backup_code(&self, valid_codes: &[String], code: &str) -> (bool, Vec<String>) {
        // Check if the code is in the list of valid codes
        let is_valid = valid_codes.contains(&code.to_string());

        // If valid, remove the code from the list
        let new_codes = if is_valid {
            valid_codes.iter()
                .filter(|c| *c != code)
                .cloned()
                .collect()
        } else {
            valid_codes.to_vec()
        };

        (is_valid, new_codes)
    }
}

#[async_trait]
impl AuthProvider for MultiFactorAuthProvider {
    fn auth_method(&self) -> AuthMethod {
        AuthMethod::MultiFactorAuth
    }

    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // First, authenticate with the base provider
        let base_token = self.base_provider.authenticate(credentials).await?;

        // Check if MFA is required
        let mfa_required = credentials.parameters.get("mfa_required").map(|v| v == "true").unwrap_or(false);

        if !mfa_required {
            // If MFA is not required, return the base token
            return Ok(base_token);
        }

        // Get the MFA factors
        let factors_json = credentials.parameters.get("mfa_factors").ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "MFA factors not provided for multi-factor authentication"
            )
        })?;

        let factors: Vec<MfaFactor> = serde_json::from_str(factors_json).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::Parse,
                &format!("Failed to parse MFA factors: {}", e)
            )
        })?;

        // Get the MFA responses
        let responses_json = credentials.parameters.get("mfa_responses").ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                "MFA responses not provided for multi-factor authentication"
            )
        })?;

        let responses: HashMap<String, String> = serde_json::from_str(responses_json).map_err(|e| {
            crate::error::Error::new(
                crate::error::ErrorKind::Parse,
                &format!("Failed to parse MFA responses: {}", e)
            )
        })?;

        // Verify each factor
        let mut verified_factors = 0;

        for factor in &factors {
            if !factor.enabled {
                continue;
            }

            let response = responses.get(&factor.id);

            if let Some(response) = response {
                let verified = match factor.factor_type {
                    MfaFactorType::Totp => {
                        let secret = factor.config.get("secret").ok_or_else(|| {
                            crate::error::Error::new(
                                crate::error::ErrorKind::Configuration,
                                "TOTP secret not configured"
                            )
                        })?;

                        let time_step = factor.config.get("time_step")
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(30);

                        self.verify_totp(secret, response, time_step)?
                    },
                    MfaFactorType::BackupCodes => {
                        let codes_json = factor.config.get("codes").ok_or_else(|| {
                            crate::error::Error::new(
                                crate::error::ErrorKind::Configuration,
                                "Backup codes not configured"
                            )
                        })?;

                        let codes: Vec<String> = serde_json::from_str(codes_json).map_err(|e| {
                            crate::error::Error::new(
                                crate::error::ErrorKind::Parse,
                                &format!("Failed to parse backup codes: {}", e)
                            )
                        })?;

                        let (is_valid, _) = self.verify_backup_code(&codes, response);
                        is_valid
                    },
                    // Other factor types would be implemented here
                    _ => false,
                };

                if verified {
                    verified_factors += 1;
                }
            }
        }

        // Check if enough factors were verified
        if verified_factors < self.config.required_factors {
            return Err(crate::error::Error::new(
                crate::error::ErrorKind::Authentication,
                &format!("Not enough MFA factors verified. Required: {}, Verified: {}", 
                    self.config.required_factors, verified_factors)
            ));
        }

        // If all required factors were verified, return the base token with MFA information
        let mut token = base_token;
        token.properties.insert("mfa_verified".to_string(), "true".to_string());
        token.properties.insert("mfa_verified_factors".to_string(), verified_factors.to_string());

        Ok(token)
    }

    async fn refresh_token(&self, token: &AuthToken) -> Result<AuthToken> {
        // Refresh the base token
        let refreshed_token = self.base_provider.refresh_token(token).await?;

        // Preserve MFA information
        let mut new_token = refreshed_token;
        if let Some(mfa_verified) = token.properties.get("mfa_verified") {
            new_token.properties.insert("mfa_verified".to_string(), mfa_verified.clone());
        }

        if let Some(mfa_verified_factors) = token.properties.get("mfa_verified_factors") {
            new_token.properties.insert("mfa_verified_factors".to_string(), mfa_verified_factors.clone());
        }

        Ok(new_token)
    }

    async fn revoke_token(&self, token: &AuthToken) -> Result<()> {
        // Revoke the base token
        self.base_provider.revoke_token(token).await
    }

    async fn validate_token(&self, token: &AuthToken) -> Result<bool> {
        // Validate the base token
        let is_valid = self.base_provider.validate_token(token).await?;

        // Check MFA verification if required
        if is_valid {
            let mfa_verified = token.properties.get("mfa_verified")
                .map(|v| v == "true")
                .unwrap_or(false);

            if !mfa_verified {
                return Ok(false);
            }

            let verified_factors = token.properties.get("mfa_verified_factors")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);

            if verified_factors < self.config.required_factors {
                return Ok(false);
            }
        }

        Ok(is_valid)
    }
}

/// Authenticated service wrapper
pub struct AuthenticatedService<T: crate::integration::interfaces::ExternalService> {
    /// The wrapped service
    service: T,

    /// Authentication service
    auth_service: std::sync::Arc<AuthService>,

    /// Credentials ID
    credentials_id: String,

    /// Current authentication token
    token: std::sync::RwLock<Option<AuthToken>>,

    /// Authentication method
    auth_method: AuthMethod,
}

impl<T: crate::integration::interfaces::ExternalService> AuthenticatedService<T> {
    /// Create a new authenticated service
    pub fn new(
        service: T, 
        auth_service: std::sync::Arc<AuthService>, 
        credentials_id: String,
        auth_method: AuthMethod
    ) -> Self {
        Self {
            service,
            auth_service,
            credentials_id,
            token: std::sync::RwLock::new(None),
            auth_method,
        }
    }

    /// Get the current authentication token, authenticating if necessary
    pub async fn get_token(&self) -> Result<AuthToken> {
        // Check if we have a token
        let token_opt = {
            let token_guard = self.token.read().map_err(|_| {
                crate::error::Error::new(
                    crate::error::ErrorKind::Internal,
                    "Failed to acquire read lock on token"
                )
            })?;

            token_guard.clone()
        };

        // If we have a token, check if it's valid
        if let Some(token) = token_opt {
            if !token.is_expired() {
                // Validate the token
                let is_valid = self.auth_service.validate_token(&self.auth_method, &token).await?;

                if is_valid {
                    return Ok(token);
                }
            }

            // If the token is expired or invalid, try to refresh it
            if let Some(refresh_token) = &token.refresh_token {
                if !refresh_token.is_empty() {
                    match self.auth_service.refresh_token(&self.auth_method, &token).await {
                        Ok(new_token) => {
                            // Update the token
                            let mut token_guard = self.token.write().map_err(|_| {
                                crate::error::Error::new(
                                    crate::error::ErrorKind::Internal,
                                    "Failed to acquire write lock on token"
                                )
                            })?;

                            *token_guard = Some(new_token.clone());

                            return Ok(new_token);
                        },
                        Err(_) => {
                            // Refresh failed, we'll need to re-authenticate
                        }
                    }
                }
            }
        }

        // If we don't have a token, or it's expired and can't be refreshed, authenticate
        let token = self.auth_service.authenticate(&self.credentials_id).await?;

        // Update the token
        let mut token_guard = self.token.write().map_err(|_| {
            crate::error::Error::new(
                crate::error::ErrorKind::Internal,
                "Failed to acquire write lock on token"
            )
        })?;

        *token_guard = Some(token.clone());

        Ok(token)
    }

    /// Get the wrapped service
    pub fn service(&self) -> &T {
        &self.service
    }
}

#[async_trait]
impl<T: crate::integration::interfaces::ExternalService + Send + Sync> crate::integration::interfaces::ExternalService for AuthenticatedService<T> {
    fn id(&self) -> &str {
        self.service.id()
    }

    fn name(&self) -> &str {
        self.service.name()
    }

    async fn capabilities(&self) -> Result<crate::integration::interfaces::ServiceCapabilities> {
        // Get the token
        let _token = self.get_token().await?;

        // Call the wrapped service
        self.service.capabilities().await
    }

    async fn status(&self) -> Result<crate::integration::interfaces::ServiceStatus> {
        // Try to get a token, but don't fail if we can't
        let token_result = self.get_token().await;

        // If we couldn't get a token, return Unavailable
        if token_result.is_err() {
            return Ok(crate::integration::interfaces::ServiceStatus::Unavailable);
        }

        // Call the wrapped service
        self.service.status().await
    }

    async fn initialize(&self) -> Result<()> {
        // Get the token
        let _token = self.get_token().await?;

        // Call the wrapped service
        self.service.initialize().await
    }

    async fn terminate(&self) -> Result<()> {
        // Call the wrapped service
        // Note: We don't need a token for termination
        self.service.terminate().await
    }
}

/// Authentication service for managing credentials and authentication
pub struct AuthService {
    provider_factory: AuthProviderFactory,
    credential_store: Box<dyn CredentialStore>,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(provider_factory: AuthProviderFactory, credential_store: Box<dyn CredentialStore>) -> Self {
        Self {
            provider_factory,
            credential_store,
        }
    }

    /// Authenticate with an external service using stored credentials
    pub async fn authenticate(&self, credentials_id: &str) -> Result<AuthToken> {
        let credentials = self.credential_store.get_credentials(credentials_id)?;

        let provider = self.provider_factory.get_provider(&credentials.method).ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Configuration,
                &format!("No authentication provider available for method: {}", credentials.method)
            )
        })?;

        provider.authenticate(&credentials).await
    }

    /// Refresh an authentication token
    pub async fn refresh_token(&self, method: &AuthMethod, token: &AuthToken) -> Result<AuthToken> {
        let provider = self.provider_factory.get_provider(method).ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Configuration,
                &format!("No authentication provider available for method: {}", method)
            )
        })?;

        provider.refresh_token(token).await
    }

    /// Validate an authentication token
    pub async fn validate_token(&self, method: &AuthMethod, token: &AuthToken) -> Result<bool> {
        let provider = self.provider_factory.get_provider(method).ok_or_else(|| {
            crate::error::Error::new(
                crate::error::ErrorKind::Configuration,
                &format!("No authentication provider available for method: {}", method)
            )
        })?;

        provider.validate_token(token).await
    }

    /// Store credentials
    pub fn store_credentials(&self, credentials: &Credentials) -> Result<()> {
        self.credential_store.store_credentials(credentials)
    }

    /// Get credentials by ID
    pub fn get_credentials(&self, id: &str) -> Result<Credentials> {
        self.credential_store.get_credentials(id)
    }

    /// Get credentials for a specific service
    pub fn get_credentials_for_service(&self, service_id: &str) -> Result<Vec<Credentials>> {
        self.credential_store.get_credentials_for_service(service_id)
    }

    /// Delete credentials
    pub fn delete_credentials(&self, id: &str) -> Result<()> {
        self.credential_store.delete_credentials(id)
    }

    /// Update credentials
    pub fn update_credentials(&self, credentials: &Credentials) -> Result<()> {
        self.credential_store.update_credentials(credentials)
    }

    /// Get the list of supported authentication methods
    pub fn supported_methods(&self) -> Vec<AuthMethod> {
        self.provider_factory.supported_methods()
    }

    /// Create an authenticated service wrapper
    pub fn create_authenticated_service<T: crate::integration::interfaces::ExternalService>(
        &self,
        service: T,
        credentials_id: &str
    ) -> Result<AuthenticatedService<T>> {
        // Get the credentials
        let credentials = self.get_credentials(credentials_id)?;

        // Create the authenticated service
        Ok(AuthenticatedService::new(
            service,
            std::sync::Arc::new(self.clone()),
            credentials_id.to_string(),
            credentials.method
        ))
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        // Note: This is a shallow clone that shares the same provider factory and credential store
        // This is intentional, as these are meant to be shared
        Self {
            provider_factory: self.provider_factory.clone(),
            credential_store: self.credential_store.clone(),
        }
    }
}
