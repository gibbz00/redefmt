# Database

## Path lookup

SQLite database files are first and foremost placed in
`$XDG_STATE_HOME/redefmt`, but can be overridden with the `REDEFMT_STATE`
variable. The resolved directory and its parent directories are created if they
do not yet exist. Per crate databases are placed a `./crate` directory to not
cause name collisions with for example `main.sqlite`.

## Identifiers

The [ID column](row_id) in SQLite is always an `i64`. It is, however, quite
improbable--if not impossible--for redefmt to ever be fill a register table
with all 4.6 quintillion rows for a single crate (autoincrement starts
halfway at 1). There's therefore more gained than lost by letting the register
function truncate the `i64` IDs to `u16`s, especially when expecting relatively
constrained target environments. Targets need now only store and send 4 bytes
for each register statement instead of 16. (Recall that both crate ID and
register ID are sent.)

Exhausing 65K rows for one crate is still improbable, but not an unrealistic
occurrence when developing large crates over a long time period. `i64` to `u16`
conversions are therefore checked. Any failure to do so will throw a compiler
error with instructions to clear the database.

To sumamrize: As long as a project isn't using more than 65K crate dependencies,
with no dependency using more than 65K register statements, then you're good
to go.

## Caching

The write and print statement tables are usually NoSQL document-like by
containing the columns `id`, `hash`, `json` only. It is the crate ID from
`main.sqlite` and this table ID that is returned by the proc macro to later be
written with the logger, before the printer reads the ID pair and looks up the
interned string and formatting options stored in the DB.

Register proc macros will first hash the data do be inserted and then check if
that hash exists in the indexed `hash` column of the corresponding table. If
none, then the data is serialized to `json`, and a new record is inserted. If
some rows are returned, then they are iterated over to make sure that the new
json actually differs from the returned json. Hash collisions are a possibility
after all. It's reason for why the `hash` column index can not be made unique,
and one of the reasons for why it would be inappropriate to store the hash
directly in the primary key.

## Concurrency

Proc macros aren't meant to be stateful so some challenges will arise by forcing
to be so.

The database must support multiple concurrent connections since crates and proc
macros are executed in parallel. Sqlite3 should support this rather well in [WAL
mode] and [PRAGMA synchronous] set to normal.

Lock contention is further mitigated by creating one database per crate. Each
proc macro begins by checking if a database for a crate exists, if not, it
is created and given an ID in a `crates` table in the `main.sqlite` database.
If it does exist from before, then the its ID is retrieved by name from the
`main.sqlite` database, warranting an index on the name column.

Crate names are provided by calling `std::env::var("CARGO_PKG_NAME")` in the proc macro itself.

[WAL mode]: https://www.sqlite.org/wal.html
[PRAGMA synchronous]: https://www.sqlite.org/pragma.html#pragma_synchronous
[Checkpointing]: https://www.sqlite.org/wal.html#ckpt
[row_id]: https://www.sqlite.org/lang_createtable.html#rowid
