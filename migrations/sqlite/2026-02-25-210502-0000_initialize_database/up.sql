-- Your SQL goes here

CREATE TABLE users (
    pub_key BLOB PRIMARY KEY,
    name TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    signature BLOB NOT NULL,
    address TEXT NOT NULL,
    trust INTEGER NOT NULL
) STRICT;

CREATE TABLE posts (
    signature BLOB PRIMARY KEY,
    source BLOB NOT NULL,
    topic BLOB NOT NULL,
    timestamp INTEGER NOT NULL,
    content TEXT NOT NULL,
    received_at INTEGER NOT NULL,
    -- FOREIGN KEY(source) REFERENCES users(pub_key)
) STRICT;

-- ==============================================================================
--                                    Index
-- ==============================================================================

-- ==================== Manga ====================
CREATE TABLE mangas(
    hash BLOB PRIMARY KEY,
    title TEXT NOT NULL,
    release_date INTEGER NOT NULL,
    source BLOB NOT NULL,
    received_at INTEGER NOT NULL,
    signature BLOB NOT NULL,
    -- FOREIGN KEY(source) REFERENCES users(pub_key)
) STRICT;

CREATE TABLE manga_chapters(
    signature BLOB PRIMARY KEY,
    source BLOB NOT NULL,
    index_hash BLOB NOT NULL,
    timestamp INTEGER NOT NULL,
    magnet_link TEXT NOT NULL,
    -- FOREIGN KEY(source) REFERENCES users(pub_key)
    -- FOREIGN KEY(index_hash) REFERENCES mangas(hash)
) STRICT;

CREATE TABLE manga_chapters_entries(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_signature BLOB NOT NULL,

    title TEXT NOT NULL,
    enumeration REAL NOT NULL,
    path TEXT NOT NULL,
    progress REAL NOT NULL,
    language TEXT NOT NULL,
    -- FOREIGN KEY(chapter_signature) REFERENCES manga_chapters(signature)
) STRICT;

CREATE TABLE manga_follows(
    hash BLOB PRIMARY KEY,
    notify INTEGER NOT NULL,
    -- FOREIGN KEY(hash) REFERENCES mangas(hash)
) STRICT;
