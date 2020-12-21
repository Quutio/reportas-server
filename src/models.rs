use crate::schema::reports;

#[derive(Queryable, Debug, PartialEq)]
pub struct Report {
    pub id: i32,

    pub reporter: String,
    pub reported: String,

    pub description: String,
}

#[derive(Insertable)]
#[table_name = "reports"]
pub struct NewReport<'a> {
    pub reporter: &'a str,
    pub reported: &'a str,
    pub description: &'a str,
}
