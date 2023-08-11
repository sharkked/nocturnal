use nocturnal::{db::DBConnection, schema::User};
use rocket::{State, serde::json::Json};
#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/users/<username>")]
async fn get_user(username: &str, db: &State<DBConnection>) -> Json<Option<User>> {
    let user = db.find_user_by_username(username).await.unwrap();
    Json(user)
}

#[get("/users/@me")]
async fn get_current_user(db: &State<DBConnection>) -> Json<Option<User>> {
    // get currently authenticated user by session

    // let user = db.find_user_by_username(username).await.unwrap();
    // Json(user)
    todo!()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let db = DBConnection::init().await.unwrap();
    rocket::build()
        .manage(db)
        .mount("/", routes![index])
        .mount("/", routes![get_user])
        .launch()
        .await?;

    Ok(())
}
