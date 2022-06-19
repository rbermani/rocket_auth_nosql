use super::rand_string;
use crate::db::DBConnection;
use crate::prelude::*;
use mongodb::bson::{oid::ObjectId};
use mongodb::{Client, options::ClientOptions};

impl Users {
    /// Opens a redis connection. It allows for sessions to be stored persistently across
    /// different launches. Note that persistent sessions also require a `secret_key` to be set in the [Rocket.toml](https://rocket.rs/v0.5-rc/guide/configuration/#configuration) configuration file.
    /// ```rust,
    /// # use rocket_auth_nosql::{Users, Error};
    /// # async fn main() -> Result<(), Error> {
    /// let mut users = Users::open_mongodb(DATABASE_URL, DATABASE).await?;
    /// users.open_redis("redis://127.0.0.1/")?;
    ///
    /// rocket::build()
    ///     .manage(users)
    ///     .launch();
    ///
    /// # Ok(()) }
    /// ```
    #[cfg(feature = "redis")]
    #[throws(Error)]
    pub fn open_redis(&mut self, path: impl redis::IntoConnectionInfo) {
        let client = redis::Client::open(path)?;
        self.sess = Box::new(client);
    }
    /// It creates a `Users` instance by connecting  it to a mongdb database.
    ///
    /// ```rust
    /// # use rocket_auth::{Error, Users};
    /// # async fn func(DATABASE_URL: &str) -> Result<(), Error> {
    /// let users = Users::open_mongodb(DATABASE_URL, DATABASE).await?;
    ///
    /// rocket::build()
    ///     .manage(users)
    ///     .launch();
    /// # Ok(()) }
    ///
    /// ```
    #[throws(Error)]
    pub async fn open_mongodb(path: &str, database: &str) -> Self {
        let client_options = ClientOptions::parse(path).await?;
        let client = Client::with_options(client_options)?;
        let conn = client.database(database).clone();

        let users: Users = conn.into();
        users
    }
    /// It queries a user by their email.
    /// ```
    /// # use rocket::{State, get};
    /// # use rocket_auth_nosql::{Error, Users};
    /// #[get("/user-information/<email>")]
    /// async fn user_information(email: String, users: &State<Users>) -> Result<String, Error> {
    ///        
    ///     let user = users.get_by_email(&email).await?;
    ///     Ok(format!("{:?}", user))
    /// }
    /// # fn main() {}
    /// ```
    #[throws(Error)]
    pub async fn get_by_email(&self, email: &str) -> User {
        self.conn.get_user_by_email(email).await?
    }

    /// It queries a user by their id.
    /// ```
    /// # use rocket::{State, get};
    /// # use rocket_auth_nosql::{Error, Users};
    /// # #[get("/user-information/<id>")]
    /// # async fn user_information(id: String, users: &State<Users>) -> Result<(), Error> {
    ///  let user = users.get_by_id(3).await?;
    ///  format!("{:?}", user);
    /// # Ok(())
    /// # }
    /// # fn main() {}
    /// ```
    #[throws(Error)]
    pub async fn get_by_id(&self, user_id: ObjectId) -> User {
        self.conn.get_user_by_id(user_id).await?
    }

    #[throws(Error)]
    pub async fn get_all_users(&self) -> Vec<User> {
        self.conn.get_all_users().await
    }
    /// Inserts a new user in the database. It will fail if the user already exists.
    /// ```rust
    /// # use rocket::{State, get};
    /// # use rocket_auth_nosql::{Error, Users};
    /// #[get("/create_admin/<email>/<password>")]
    /// async fn create_admin(email: String, password: String, users: &State<Users>) -> Result<String, Error> {
    ///     users.create_user(&email, &password, true).await?;
    ///     Ok("User created successfully".into())
    /// }
    /// # fn main() {}
    /// ```
    #[throws(Error)]
    pub async fn create_user(&self, email: &str, password: &str, is_admin: bool) {
        let password = password.as_bytes();
        let salt = rand_string(30);
        let config = argon2::Config::default();
        let hash = argon2::hash_encoded(password, salt.as_bytes(), &config).unwrap();
        self.conn.create_user(email, &hash, is_admin).await?;
    }

    /// Deletes a user from de database. Note that this method won't delete the session.
    /// To do that use [`Auth::delete`](crate::Auth::delete).
    /// ```
    /// #[get("/delete_user/<id>")]
    /// async fn delete_user(id: i32, users: &State<Users>) -> Result<String> {
    ///     users.delete(id).await?;
    ///     Ok("The user has been deleted.")
    /// }
    /// ```
    #[throws(Error)]
    pub async fn delete(&self, id: ObjectId) {
        self.sess.remove(id)?;
        self.conn.delete_user_by_id(id).await?;
    }

    /// Modifies a user in the database.
    /// ```
    /// # use rocket_auth_nosql::{Users, Error};
    /// # async fn func(users: Users) -> Result<(), Error> {
    /// let mut user = users.get_by_id(4).await?;
    /// user.set_email("new@email.com");
    /// user.set_password("new password");
    /// users.modify(&user).await?;
    /// # Ok(())}
    /// ```
    #[throws(Error)]
    pub async fn modify(&self, user: &User) {
        self.conn.update_user(user).await?;
    }
}

/// A `Users` instance can also be created from a database connection.
/// ```rust
/// # use rocket_auth_nosql::{Users, Error};
/// use mongodb::{Client, options::ClientOptions};
/// # async fn func() -> Result<(), Error> {
/// let client_options = ClientOptions::parse("mongdb://localhost:27017").await?;
/// let client = Client::with_options(client_options)?;
/// let conn = client.database(DATABASE).clone();
/// let users: Users = conn.clone().into();
/// # Ok(())}
/// ```

impl<Conn: 'static + DBConnection> From<Conn> for Users {
    fn from(db: Conn) -> Users {
        Users {
            conn: Box::from(db),
            sess: Box::new(chashmap::CHashMap::new()),
        }
    }
}

/// Additionally, `Users` can be created from a tuple,
/// where the first element is a database connection, and the second is a redis connection.
/// ```rust
/// # use rocket_auth_nosql::{Users, Error};
/// # extern crate redis;
/// # async fn func(postgres_path: &str, redis_path: &str) -> Result<(), Error> {
/// let (db_client, connection) = tokio_postgres::connect(postgres_path, NoTls).await?;
/// let redis_client = redis::Client::open(redis_path)?;
///
/// let users: Users = (db_client, redis_client).into();
/// # Ok(())}
/// ```
impl<T0: 'static + DBConnection, T1: 'static + SessionManager> From<(T0, T1)> for Users {
    fn from((db, ss): (T0, T1)) -> Users {
        Users {
            conn: Box::from(db),
            sess: Box::new(ss),
        }
    }
}
