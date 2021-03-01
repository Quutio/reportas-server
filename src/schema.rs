table! {
    reports (id) {
        id -> Int8,
        active -> Bool,
        timestamp -> Int8,
        reporter -> Text,
        reported -> Text,
        description -> Text,
    }
}
