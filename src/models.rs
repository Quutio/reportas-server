use crate::schema::reports;

#[derive(Queryable, Debug, PartialEq)]
pub struct Report {
    pub id: i64,
    pub active: bool,
    pub timestamp: i64,

    pub reporter: String,
    pub reported: String,

    pub description: String,
}

#[derive(Insertable)]
#[table_name = "reports"]
pub struct NewReport<'a> {
    pub active: bool,
    pub timestamp: i64,
    pub reporter: &'a str,
    pub reported: &'a str,
    pub description: &'a str
}
