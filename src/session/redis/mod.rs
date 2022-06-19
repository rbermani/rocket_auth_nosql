use super::SessionManager;
use crate::prelude::*;

use redis::{Client, Commands};
use mongodb::bson::oid::ObjectId;

const YEAR_IN_SECS: usize = 365 * 60 * 60 * 24;

impl SessionManager for Client {
    #[throws(Error)]
    fn insert(&self, id: ObjectId, key: String) {
        let mut cnn = self.get_connection()?;
        cnn.set_ex(&id.bytes(), key, YEAR_IN_SECS)?;
    }
    #[throws(Error)]
    fn insert_for(&self, id: ObjectId, key: String, time: Duration) {
        let mut cnn = self.get_connection()?;
        cnn.set_ex(&id.bytes(), key, time.as_secs() as usize)?;
    }
    #[throws(Error)]
    fn remove(&self, id: ObjectId) {
        let mut cnn = self.get_connection()?;
        cnn.del(&id.bytes())?;
    }
    #[throws(as Option)]
    fn get(&self, id: ObjectId) -> String {
        let mut cnn = self.get_connection().ok()?;
        let key = cnn.get(&id.bytes()).ok()?;
        key
    }
    #[throws(Error)]
    fn clear_all(&self) {
        let mut cnn = self.get_connection()?;
        redis::Cmd::new().arg("FLUSHDB").execute(&mut cnn);
    }
    #[throws(Error)]
    fn clear_expired(&self) {}
}
