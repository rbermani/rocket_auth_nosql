use super::AuthKey;
use super::SessionManager;
use crate::prelude::*;
use chashmap::CHashMap;
use mongodb::bson::{oid::ObjectId};

impl SessionManager for CHashMap<ObjectId, AuthKey> {
    #[throws(Error)]
    fn insert(&self, id: ObjectId, key: String) {
        self.insert(id, key.into());
    }

    #[throws(Error)]
    fn remove(&self, id: ObjectId) {
        self.remove(&id);
    }

    fn get(&self, id: ObjectId) -> Option<String> {
        let key = self.get(&id)?;
        Some(key.secret.clone())
    }

    #[throws(Error)]
    fn clear_all(&self) {
        self.clear();
    }

    #[throws(Error)]
    fn insert_for(&self, id: ObjectId, key: String, time: Duration) {
        let key = AuthKey {
            expires: time.as_secs() as i64,
            secret: key,
        };
        self.insert(id, key);
    }

    #[throws(Error)]
    fn clear_expired(&self) {
        let time = now();
        self.retain(|_, auth_key| auth_key.expires > time);
    }
}

