# 6. Restoring backups

- Status: Accepted
- Date: 2026-10-15

## Context

ADR 0005 makes `Store::open` refuse a damaged, too-new or non-Cards database, and noted that recovery meant restoring a file by hand. Doing that by hand is risky. SQLite keeps recent commits in `cards.sqlite3-wal` until a checkpoint. If a backup is copied over `cards.sqlite3` while that file is still there, SQLite replays the old WAL into the copied-in database. That either corrupts it or brings back data from the database being replaced. SPEC §1 puts data safety first, so the core needs a restore that cannot do that and never destroys the current database.

## Decision

**Listing.** `list_backups(data_dir)` returns `BackupInfo` (path, creation time, size), newest first.
- The time comes from the UTC stamp in the file name, not from mtime, which copying or touching a file changes.
- A file is listed if it has a well-formed backup name, is a regular file (not a symlink) and starts with the 16-byte SQLite header. This filter is cheap on purpose. Running a full integrity check on 14 files every time the list is shown would get slow as the database grows. The full check happens at restore time.
- `BackupInfo` has private fields, so only `list_backups` creates one. `restore_backup` still checks that the path is a regular, well-named file directly in `backups/`. It only reads files Cards made itself.

**Validate the copy, not the original.** The backup is byte-copied to `cards.sqlite3.restore.checking` in the data directory (mode `0600`, synced). Only after it passes the checks below is it renamed to `cards.sqlite3.restore.partial`. That name therefore always means "a validated copy waiting to be installed", and an attempt that fails never deletes one left by an earlier interrupted restore (see below). Any leftover copy and any `-wal`/`-shm`/`-journal` next to it are removed first. The copy must be in rollback-journal mode (header bytes 18 and 19 both 1), as every backup is. A WAL-mode file would make even a read-only connection create and read `-wal`/`-shm` files beside it, so the checks would no longer be on the file alone. The checks then run on that copy, through a read-only connection with `trusted_schema=OFF`. The file that gets installed is therefore exactly the file that was checked, and the backup itself is never opened by SQLite. The copy must pass:
1. `PRAGMA integrity_check`;
2. `1 ≤ user_version ≤` the last migration this build knows;
3. an exact schema match: the set of `(type, name)` of every table, index, trigger and view equals that of an in-memory database built by the real migrations up to the same version. SQLite's own `sqlite_*` objects (`sqlite_stat1`, `sqlite_sequence`, autoindexes) are ignored, because they come and go with use. SQL text is not compared, because SQLite may store it differently across versions. Names are enough to tell our schema from anything else, and integrity was checked already. The limitation: a backup whose append-only trigger body was rewritten would still pass. That is acceptable because restore only reads files Cards wrote into its own `backups/` directory. If backups ever come from untrusted places (an import), compare normalised SQL too.

If a check fails, the scratch copy is deleted and `Error::RestoreRefused` is returned, wrapping the reason. The UI can then tell "this backup is bad" apart from "your database is bad". Nothing else has been touched.

**Move aside, never delete.** The current `cards.sqlite3` and its `-wal`, `-shm` and `-journal` files are renamed to `cards.sqlite3.replaced-YYYYMMDD-HHMMSS` plus the same suffixes. The name gets `-2`, `-3`… if any of those names is taken. Each file gets mode `0600`. SQLite looks for `<name>-wal` next to `<name>`, so the moved-aside set still opens as one consistent database, including rows not yet checkpointed into the main file. The current database is never opened, so a restore works when it is damaged or is not a database at all.

**Main file first, plus guards in `Store::open`.** Several renames cannot be atomic together. The main file is renamed first. From that moment until the copy is installed, there is no `cards.sqlite3`. A plain `Store::open` would then create an empty database. That is bad in two ways. A leftover `-wal` would be replayed into it. And even without one, the user would see an empty app and start using it, while its daily backups pushed the real ones out of the 14 kept. So `Store::open` refuses when the main file is missing and either:
- a `-wal` or `-journal` is there (`Error::OrphanedSidecar`); or
- the validated copy `cards.sqlite3.restore.partial` is there (`Error::InterruptedRestore`).

For this reason, once moving aside has started, the copy is never deleted on an error. Running restore again replaces the copy, moves any stray files aside and finishes. The opposite rename order (sidecars first) was rejected: a crash in between would leave a main file that opens cleanly but silently lacks its latest commits. A stale `-shm` alone is harmless, since it is only an index that SQLite rebuilds.

**The open store is handed over.** `restore_backup` takes `current: Option<Store>` by value and drops it before touching files. The compiler then enforces that the caller cannot keep writing through a connection to the moved-aside file. Closing also checkpoints its WAL. `None` covers a database that could not be opened.

**Install.** `restore_backup` checks that no database file or sidecar is left at `cards.sqlite3`. It then renames the copy into place and syncs the directory. Finally it opens the database with the normal `Store::open`, which migrates an older backup after making the usual pre-migration backup.

## Consequences

- Recovering no longer means copying files by hand. The README tells users to restore through Cards, once the UI exists.
- `replaced-*` files are never pruned. They only appear when the user restores, which is rare, and deleting them automatically would go against "never delete the current database". Cleaning them up can come later, in the UI.
- There is no cross-process lock. Restore relies on the app being single-instance (GApplication), so that no other process has the file open. For the same reason, the "is this name free" checks before each rename are not atomic with the rename, and `File::open` follows a symlink swapped in after the regular-file check. Closing these races needs `renameat2(RENAME_NOREPLACE)` and `O_NOFOLLOW`, which means a syscall crate. That is not worth adding while there is a single instance.
- The moved-aside database is not in `list_backups`, so the restore UI cannot undo a restore yet. A follow-up for that slice: either list `replaced-*` files, or make a normal backup of the current database before moving it, when it is readable.
- A restore needs free space for one more copy of the backup in the data directory.
- A crash while the copy is being made or checked leaves `cards.sqlite3.restore.checking` behind. It is harmless: `Store::open` ignores it, and the next restore removes it.
