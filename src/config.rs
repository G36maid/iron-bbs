use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub web_port: u16,
    pub ssh_port: u16,
    pub ssh_host_key_path: String,
}

impl Config {
    pub fn from_env() -> crate::Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            web_port: env::var("WEB_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("WEB_PORT must be a valid port number"),
            ssh_port: env::var("SSH_PORT")
                .unwrap_or_else(|_| "2222".to_string())
                .parse()
                .expect("SSH_PORT must be a valid port number"),
            ssh_host_key_path: env::var("SSH_HOST_KEY_PATH")
                .unwrap_or_else(|_| "./ssh_host_key".to_string()),
        })
    }

    pub fn web_addr(&self) -> String {
        format!("0.0.0.0:{}", self.web_port)
    }

    pub fn ssh_addr(&self) -> String {
        format!("0.0.0.0:{}", self.ssh_port)
    }
}
