use actix_web_httpauth::extractors::basic::BasicAuth;

#[derive(Clone)]
pub struct Auth {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Auth {
    pub fn check_credentials(&self, credentials: BasicAuth) -> bool {
        if !self.has_credentials() {
            return true;
        }

        let username = self.username.clone();
        let password = self.password.clone();

        if let (Some(username), Some(password)) = (username, password) {
            credentials.user_id() == username
                && credentials.password().unwrap_or_default() == password
        } else {
            true
        }
    }
    pub fn has_credentials(&self) -> bool {
        let username = self.username.clone();
        let password = self.password.clone();
        username.is_some() && password.is_some()
    }
}
