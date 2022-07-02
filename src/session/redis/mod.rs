use super::SessionManager;
use crate::prelude::*;

use redis::{Client, Commands};
use mongodb::bson::oid::ObjectId;

const YEAR_IN_SECS: usize = 365 * 60 * 60 * 24;

impl SessionManager for Client {

    fn insert(&self, id: ObjectId, key: String) -> Result<()> {
        let mut cnn = self.get_connection()?;
        cnn.set_ex(&id.bytes(), key, YEAR_IN_SECS)?;
        Ok(())
    }

    fn insert_for(&self, id: ObjectId, key: String, time: Duration) -> Result<()> {
        let mut cnn = self.get_connection()?;
        cnn.set_ex(&id.bytes(), key, time.as_secs() as usize)?;
        Ok(())
    }

    fn remove(&self, id: ObjectId) -> Result<()> {
        let mut cnn = self.get_connection()?;
        cnn.del(&id.bytes())?;
        Ok(())
    }

    fn get(&self, id: ObjectId) -> Option<String> {
        let mut cnn = self.get_connection().ok()?;
        let key = cnn.get(&id.bytes()).ok()?;
        Some(key)
    }

    fn clear_all(&self) -> Result<()> {
        let mut cnn = self.get_connection()?;
        redis::Cmd::new().arg("FLUSHDB").execute(&mut cnn);
        Ok(())
    }

    fn clear_expired(&self) -> Result<()> { Ok(())}
}
