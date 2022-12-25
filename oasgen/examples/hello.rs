#![allow(unused)]
use oasgen::OaSchema;

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
    let x = Box::new(5) as Box<dyn std::any::Any>;
    let y = Box::new("hi") as Box<dyn std::any::Any>;
    let z = vec![x, y];
    // let s = User::schema();
    // println!("{:#?}", s);
}