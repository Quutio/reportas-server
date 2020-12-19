#[derive(Queryable, Debug, PartialEq)]
pub struct Report {
    pub id: i32,

    pub reporter: String,
    pub reported: String,

    pub description: String,
}
