#[derive(Queryable, Debug, PartialEq)]
pub struct Report {
    pub id: i64,

    pub reporter: String,
    pub reported: String,

    pub desc: String,
}
