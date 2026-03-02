// @generated automatically by Diesel CLI.

diesel::table! {
    manga_chapters (signature) {
        signature -> Binary,
        source -> Binary,
        index_hash -> Binary,
        timestamp -> BigInt,
        magnet_link -> Text,
    }
}

diesel::table! {
    manga_chapters_entries (id) {
        id -> Nullable<Integer>,
        chapter_signature -> Binary,
        title -> Text,
        enumeration -> Float,
        path -> Text,
        progress -> Float,
        language -> Text,
    }
}

diesel::table! {
    manga_follows (hash) {
        hash -> Binary,
        notify -> Integer,
    }
}

diesel::table! {
    mangas (hash) {
        hash -> Binary,
        title -> Text,
        release_date -> Integer,
        source -> Binary,
        received_at -> BigInt,
        signature -> Binary,
    }
}

diesel::table! {
    posts (signature) {
        signature -> Binary,
        source -> Binary,
        topic -> Binary,
        timestamp -> BigInt,
        content -> Text,
        received_at -> Integer,
    }
}

diesel::table! {
    users (pub_key) {
        pub_key -> Binary,
        name -> Text,
        timestamp -> BigInt,
        signature -> Binary,
        address -> Text,
        trust -> Integer,
    }
}

diesel::joinable!(manga_chapters -> mangas (index_hash));
diesel::joinable!(manga_chapters -> users (source));
diesel::joinable!(manga_chapters_entries -> manga_chapters (chapter_signature));
diesel::joinable!(manga_follows -> mangas (hash));
diesel::joinable!(mangas -> users (source));
diesel::joinable!(posts -> users (source));

diesel::allow_tables_to_appear_in_same_query!(
    manga_chapters,
    manga_chapters_entries,
    manga_follows,
    mangas,
    posts,
    users,
);
