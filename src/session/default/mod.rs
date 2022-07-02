use super::AuthKey;
use super::SessionManager;
use crate::prelude::*;
use chashmap::CHashMap;
use mongodb::bson::{oid::ObjectId};

impl SessionManager for CHashMap<ObjectId, AuthKey> {

    fn insert(&self, id: ObjectId, key: String) -> Result<()> {
        self.insert(id, key.into());
        Ok(())
    }

    fn remove(&self, id: ObjectId) -> Result<()> {
        self.remove(&id);
        Ok(())
    }

    fn get(&self, id: ObjectId) -> Option<String> {
        let key = self.get(&id)?;
        Some(key.secret.clone())
    }

    fn clear_all(&self) -> Result<()> {
        self.clear();
        Ok(())
    }

    fn insert_for(&self, id: ObjectId, key: String, time: Duration) -> Result<()>  {
        let key = AuthKey {
            expires: time.as_secs() as i64,
            secret: key,
        };
        self.insert(id, key);
        Ok(())
    }

    fn clear_expired(&self) -> Result<()> {
        let time = now();
        self.retain(|_, auth_key| auth_key.expires > time);
        Ok(())
    }
}

