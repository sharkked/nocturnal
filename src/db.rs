use crate::schema::{Message, User};
use dotenv::dotenv;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{ClientOptions, ServerApi, ServerApiVersion},
    results::{DeleteResult, InsertOneResult},
    Client, Collection,
};
use std::env;
use thiserror::Error;

pub struct DBConnection {
    users: Collection<User>,
    messages: Collection<Message>,
}

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct MongoError(#[from] mongodb::error::Error);

impl DBConnection {
    pub async fn init() -> Result<Self> {
        dotenv().ok();
        let uri = env::var("MONGO_URI").expect("MONGO_URI must be set.");

        let mut client_options = ClientOptions::parse(&uri).await?;

        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        let client = Client::with_options(client_options)?;
        let db = client.database("nocturnal");

        let users: Collection<User> = db.collection("users");
        let messages: Collection<Message> = db.collection("messages");

        Ok(DBConnection { users, messages })
    }

    pub async fn ping(&self) -> Result<()> {
        self.users
            .client()
            .database("nocturnal")
            .run_command(doc! {"ping":1}, None)
            .await?;
        Ok(())
    }

    pub async fn find_user(&self, id: ObjectId) -> Result<Option<User>> {
        let user = self.users.find_one(doc! {"_id": id}, None).await?;
        Ok(user)
    }

    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = self
            .users
            .find_one(doc! {"username": username}, None)
            .await?;
        Ok(user)
    }

    pub async fn create_user(&self, new_user: User) -> Result<InsertOneResult> {
        let new_doc = User {
            id: None,
            username: new_user.username,
            displayname: new_user.displayname,
            created_at: None,
        };
        Ok(self.users.insert_one(new_doc, None).await?)
    }

    pub async fn delete_user(&self, id: ObjectId) -> Result<DeleteResult> {
        Ok(self.users.delete_one(doc! { "_id": id }, None).await?)
    }

    pub async fn create_message(&self, content: &str, author: ObjectId) -> Result<InsertOneResult> {
        let new_doc = Message {
            id: None,
            author,
            content: String::from(content),
            created_at: None,
        };
        Ok(self.messages.insert_one(new_doc, None).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::OnceCell;

    static DB: OnceCell<DBConnection> = OnceCell::const_new();

    async fn init_db() -> DBConnection {
        let mut db_connection = DBConnection::init().await.expect("Database should connect");
        let db = db_connection.users.client().database("nocturnal");
        db_connection.users = db.collection("users-test");
        db_connection.messages = db.collection("messages-test");
        return db_connection;
    }

    #[tokio::test]
    async fn db_is_connected() {
        let db = DB.get_or_init(init_db).await;
        db.ping().await.expect("Should be able to ping the db.");
    }

    #[tokio::test]
    async fn basic_user_operations() {
        let db = DB.get_or_init(init_db).await;
        let username = "evvil";
        let new_user = User {
            id: None,
            username: String::from(username),
            displayname: String::from(username),
            created_at: None,
        };
        db.create_user(new_user).await.expect("create should be ok");
        let user = db
            .find_user_by_username(username)
            .await
            .expect("find by username should be ok")
            .expect("user with this username should exist");
        let id = user.id.unwrap();
        db.find_user(id)
            .await
            .expect("find by id should be ok")
            .expect("user with this id should exist");
        db.delete_user(id).await.expect("delete should be ok");
    }
}
