CREATE TABLE type_structure_register(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    hash INTEGER NOT NULL,
    statement BLOB NOT NULL
);

CREATE INDEX type_structure_hash ON type_structure_register(hash);
