use futures::TryStreamExt;
use crate::prelude::{Result, *};

use mongodb::bson::{doc, oid::ObjectId};
use mongodb::options::IndexOptions;
use mongodb::{Database,IndexModel};
use crate::Error::UserNotFoundError;

const COLLECTION: &str = "users";

#[rocket::async_trait]
impl DBConnection for Database {
    async fn create_user(&self, email: &str, hash: &str, token: &str, is_admin: bool) -> Result<()> {
        let new_index = IndexModel::builder()
            .keys(doc!{"email": 1})
            .options(IndexOptions::builder()
                .unique(true)
                .name("email".to_string())
                .build())
            .build();
		let user_rec = User {
            id: None,
			email: email.to_string(),
			is_admin: is_admin,
            is_verified: false,
            verification_token: token.to_string(),
			password: hash.to_string(),
            prev_password: None,
            prev_password_1: None,
            prev_password_2: None
		};
        // Ensure the collection index exists for unique email values
        self.collection::<User>(COLLECTION)
            .create_index(new_index, None).await?;

		self.collection::<User>(COLLECTION)
			.insert_one(user_rec, None).await?;
        Ok(())
    }
    async fn update_user(&self, user: &User) -> Result<()> {
    self.collection::<User>(COLLECTION)
        .find_one_and_replace(doc! {
            "_id": user.id()
        },
        user,
        None,
        ).await?;
        Ok(())
    }
    async fn delete_user_by_id(&self, user_id: ObjectId) -> Result<()> {
        self.collection::<User>(COLLECTION)
        .delete_one(doc! {
            "_id": user_id
        },
        None,
        ).await?;
        Ok(())
    }
    async fn delete_user_by_email(&self, email: &str) -> Result<()> {
        self.collection::<User>(COLLECTION)
        .delete_one(doc! {
            "email": email.to_string()
        },
        None,
        ).await?;
        Ok(())
    }
    async fn get_user_by_id(&self, user_id: ObjectId) -> Result<User> {
        if let Some(user_rec) = self.collection::<User>(COLLECTION)
        .find_one(doc! {
            "_id": user_id
        },
        None,
        ).await? {
            Ok(user_rec)
        } else {
            Err(UserNotFoundError)
        }
    }
    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        if let Some(user_rec) = self.collection::<User>(COLLECTION)
        .find_one(doc! {
            "email": email.to_string()
        },
        None,
        ).await? {
            Ok(user_rec)
        } else {
            Err(UserNotFoundError)
        }
    }
    async fn get_all_users(&self) -> Vec<User> {
        let cursor = match self.collection::<User>(COLLECTION)
            .find(None,
            None).await {
                Ok(cursor) => cursor,
                Err(_) => return vec![],
            };

        cursor.try_collect().await.unwrap_or_else(|_| vec![])
    }
}
