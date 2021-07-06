table! {
    reports (id) {
        id -> Int8,
        active -> Bool,
        timestamp -> Int8,
        reporter -> Text,
        reported -> Text,
        handler -> Nullable<Text>,
        handle_ts -> Nullable<Int8>,
        comment -> Nullable<Text>,
        description -> Text,
    }
}
