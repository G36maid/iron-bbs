use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use uuid::Uuid;

pub struct AuthService;

impl AuthService {
    pub fn hash_password(password: &str) -> crate::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| crate::Error::Internal(format!("Failed to hash password: {}", e)))?
            .to_string();

        Ok(password_hash)
    }

    pub fn verify_password(password: &str, password_hash: &str) -> crate::Result<bool> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| crate::Error::Internal(format!("Invalid password hash: {}", e)))?;

        let argon2 = Argon2::default();

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub fn generate_session_token() -> String {
        Uuid::new_v4().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = AuthService::hash_password(password).unwrap();

        assert!(AuthService::verify_password(password, &hash).unwrap());
        assert!(!AuthService::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    #[ignore]
    fn generate_admin_hash() {
        let password = "admin123";
        let hash = AuthService::hash_password(password).unwrap();
        println!("\n\nPassword: {}\nHash: {}\n\n", password, hash);
    }
}
