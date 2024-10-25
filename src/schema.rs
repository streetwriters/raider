// @generated automatically by Diesel CLI.

diesel::table! {
    account (id) {
        id -> Integer,
        email -> Text,
        password -> Binary,
        recovery -> Nullable<Binary>,
        commission -> Double,
        full_name -> Nullable<Text>,
        address -> Nullable<Text>,
        country -> Nullable<Text>,
        payout_method -> Nullable<Text>,
        payout_instructions -> Nullable<Text>,
        notify_balance -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    balance (id) {
        id -> Integer,
        amount -> Double,
        currency -> Text,
        released -> Bool,
        trace -> Nullable<Text>,
        account_id -> Integer,
        tracker_id -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    payout (id) {
        id -> Integer,
        number -> Integer,
        amount -> Double,
        currency -> Text,
        status -> Text,
        account -> Nullable<Text>,
        invoice_url -> Nullable<Text>,
        account_id -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    tracker (id) {
        id -> Text,
        label -> Text,
        statistics_signups -> Integer,
        account_id -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(balance -> account (account_id));
diesel::joinable!(balance -> tracker (tracker_id));
diesel::joinable!(payout -> account (account_id));
diesel::joinable!(tracker -> account (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    account,
    balance,
    payout,
    tracker,
);
