CREATE TABLE print_register(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hash INTEGER NOT NULL,
    statement BLOB NOT NULL
);

CREATE INDEX print_hash ON print_register(hash);
