table! {
    timers (id) {
        id -> Int4,
        title -> Varchar,
        body -> Nullable<Text>,
        recurring -> Bool,
        raid_lead -> Nullable<Varchar>,
        time -> Text,
    }
}
