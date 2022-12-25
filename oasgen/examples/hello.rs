use oasgen::OaSchema;
use oasgen::core::OaSchema;

#[derive(OaSchema)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub phone: String,
    pub user_status: Option<i32>,
}

fn main() {
    let s = User::schema();
    println!("{:#?}", s);
}