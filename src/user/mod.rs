pub mod auth;
mod user_impl;
mod users;
use crate::prelude::*;
use argon2::verify_encoded as verify;
use mongodb::bson::{oid::ObjectId};

use rand::random;
pub fn rand_string(size: usize) -> String {
    (0..)
        .map(|_| random::<char>())
        .filter(|c| c.is_ascii())
        .map(char::from)
        .take(size)
        .collect()
}

impl Users {
    fn is_auth(&self, session: &Session) -> bool {
        let option = self.sess.get(session.id);
        if let Some(auth_key) = option {
            auth_key == session.auth_key
        } else {
            false
        }
    }

    #[throws(Error)]
    async fn login(&self, form: &Login) -> String {
        let form_pwd = &form.password.as_bytes();
        let user = self
            .conn
            .get_user_by_email(&form.email)
            .await
            .map_err(|_| Error::EmailDoesNotExist(form.email.clone()))?;
        let user_pwd = &user.password;
        if verify(user_pwd, form_pwd)? {
            self.set_auth_key(user.id.unwrap())?
        } else {
            throw!(Error::UnauthorizedError)
        }
    }
    #[throws(Error)]
    fn logout(&self, session: &Session) {
        if self.is_auth(session) {
            self.sess.remove(session.id)?;
        }
    }

    #[throws(Error)]
    fn set_auth_key_for(&self, user_id: ObjectId, time: Duration) -> String {
        let key = rand_string(10);
        self.sess.insert_for(user_id, key.clone(), time)?;
        key
    }

    #[throws(Error)]
    fn set_auth_key(&self, user_id: ObjectId) -> String {
        let key = rand_string(15);
        self.sess.insert(user_id, key.clone())?;
        key
    }

    #[throws(Error)]
    async fn signup(&self, form: &Signup) {
        form.validate()?;
        let email = &form.email;
        let password = &form.password;
        let result = self.create_user(email, password, false).await;
        match result {
            Ok(_) => (),
            Err(error) => {
                throw!(error)
            }
        }
    }

    #[throws(Error)]
    async fn login_for(&self, form: &Login, time: Duration) -> String {
        let form_pwd = &form.password.as_bytes();
        let user = self.conn.get_user_by_email(&form.email).await?;
        let user_pwd = &user.password;
        if verify(user_pwd, form_pwd)? {
            self.set_auth_key_for(user.id.unwrap(), time)?
        } else {
            throw!(Error::UnauthorizedError)
        }
    }
}