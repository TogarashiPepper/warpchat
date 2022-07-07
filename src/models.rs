#[derive(Queryable)]
pub struct Post {
    pub id: i32,
    pub msg: String
}

use super::schema::posts;

#[derive(Insertable)]
#[table_name="posts"]
pub struct NewPost<'a> {
    pub msg: &'a str,
}