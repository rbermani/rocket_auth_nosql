mod mongomodel;

use crate::prelude::*;
use mongodb::bson::{oid::ObjectId};

#[rocket::async_trait]
pub trait DBConnection: Send + Sync {
    async fn create_user(&self, email: &str, hash: &str, is_admin: bool) -> Result<(), Error>;
    async fn update_user(&self, user: &User) -> Result<()>;
    async fn delete_user_by_id(&self, user_id: ObjectId) -> Result<()>;
    async fn delete_user_by_email(&self, email: &str) -> Result<()>;
    async fn get_user_by_id(&self, user_id: ObjectId) -> Result<User>;
    async fn get_user_by_email(&self, email: &str) -> Result<User>;
    async fn get_all_users(&self) -> Vec<User>;
}

#[rocket::async_trait]
impl<T: DBConnection> DBConnection for std::sync::Arc<T> {
    async fn create_user(&self, email: &str, hash: &str, is_admin: bool) -> Result<(), Error> {
        T::create_user(self, email, hash, is_admin).await
    }
    async fn update_user(&self, user: &User) -> Result<()> {
        T::update_user(self, user).await
    }
    async fn delete_user_by_id(&self, user_id: ObjectId) -> Result<()> {
        T::delete_user_by_id(self, user_id).await
    }
    async fn delete_user_by_email(&self, email: &str) -> Result<()> {
        T::delete_user_by_email(self, email).await
    }
    async fn get_user_by_id(&self, user_id: ObjectId) -> Result<User> {
        T::get_user_by_id(self, user_id).await
    }
    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        T::get_user_by_email(self, email).await
    }
    async fn get_all_users(&self) -> Vec<User> {
        T::get_all_users(self).await
    }
}

#[rocket::async_trait]
impl<T: DBConnection> DBConnection for tokio::sync::Mutex<T> {
    async fn create_user(&self, email: &str, hash: &str, is_admin: bool) -> Result<(), Error> {
        self.lock().await.create_user(email, hash, is_admin).await
    }
    async fn update_user(&self, user: &User) -> Result<()> {
        self.lock().await.update_user(user).await
    }
    async fn delete_user_by_id(&self, user_id: ObjectId) -> Result<()> {
        self.lock().await.delete_user_by_id(user_id).await
    }
    async fn delete_user_by_email(&self, email: &str) -> Result<()> {
        self.lock().await.delete_user_by_email(email).await
    }
    async fn get_user_by_id(&self, user_id: ObjectId) -> Result<User> {
        self.lock().await.get_user_by_id(user_id).await
    }
    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        self.lock().await.get_user_by_email(email).await
    }
    async fn get_all_users(&self) -> Vec<User> {
        self.lock().await.get_all_users().await
    }
}

