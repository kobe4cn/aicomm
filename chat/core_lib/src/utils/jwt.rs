use jwt_simple::prelude::*;

use crate::User;
use jwt_simple::Error;
const JWT_DURATION: u64 = 30;
const JWT_ISSUER: &str = "chat_server";
const JWT_AUDIENCE: &str = "chat_web";

pub struct EncodingKey(Ed25519KeyPair);
#[allow(unused)]
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, Error> {
        let key = Ed25519KeyPair::from_pem(pem)?;
        Ok(Self(key))
    }
    pub fn sign(&self, user: impl Into<User>) -> Result<String, Error> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_days(JWT_DURATION))
            .with_issuer(JWT_ISSUER)
            .with_audience(JWT_AUDIENCE);
        self.0.sign(claims)
    }
}
impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, Error> {
        let key = Ed25519PublicKey::from_pem(pem)?;
        Ok(Self(key))
    }
    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, Error> {
        let options = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISSUER])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUDIENCE])),
            ..Default::default()
        };

        let claims = self.0.verify_token::<User>(token, Some(options))?;
        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::*;

    #[tokio::test]
    async fn jwt_sign_verify_should_work() -> Result<()> {
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");
        let ek = EncodingKey::load(encoding_pem)?;
        let dk = DecodingKey::load(decoding_pem)?;

        let user = User::new(1, "kevin yang", "kevin.yang.xgz@gamil.com");
        let token = ek.sign(user.clone())?;
        let user2 = dk.verify(&token)?;
        // assert_eq!(token, "");
        assert_eq!(user, user2);

        Ok(())
    }
}
