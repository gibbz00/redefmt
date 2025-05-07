CREATE TABLE write_register(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hash INTEGER NOT NULL,
    statement BLOB NOT NULL
);

CREATE INDEX write_hash ON write_register(hash);
