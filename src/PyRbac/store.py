# =============================================================================
# Database access for PyRbac.
# requirement-database — sqlite3 or MySQL, one connection per call, parameters only.
# requirement-python-oop — class Store and StoreError.
# =============================================================================
from __future__ import annotations

import os
import re
import sqlite3


class StoreError(Exception):
    """The database did not complete the call."""

    def __init__(self, message, nxt, logger=None):
        super().__init__(message)
        self.logger = logger
        self.nxt = nxt
        if logger is not None:
            logger.log_message("instantiated", component="StoreError")


class Store:
    """Short connections. Callers pass parameters. This class does not build SQL from names.

    sqlite3 is the default. mysql keeps the PyMySQL statements. The sqlite3
    path rewrites placeholders, NOW(), and FIND_IN_SET. It does not log SQL.
    """

    def __init__(self, settings, logger=None):
        self.logger = logger
        if logger is not None:
            logger.log_message("instantiated", component="Store")
        self.settings = settings

    def connect(self):
        """General Purpose: One new connection. The caller closes it."""
        try:
            engine = self.settings.engine()
        except Exception as exc:
            nxt = getattr(exc, "nxt", "py-rbac db-engine")
            raise StoreError(str(exc), nxt, logger=self.logger)
        if engine == "sqlite3":
            return self._connect_sqlite()
        return self._connect_mysql()

    def _connect_sqlite(self):
        try:
            path = self.settings.sqlite_path()
        except Exception as exc:
            nxt = getattr(exc, "nxt", "export PYRBAC_HOME")
            raise StoreError(str(exc), nxt, logger=self.logger)
        folder = os.path.dirname(path)
        try:
            if folder:
                os.makedirs(folder, mode=0o700, exist_ok=True)
                os.chmod(folder, 0o700)
            connection = sqlite3.connect(path)
            connection.isolation_level = "DEFERRED"
            connection.row_factory = sqlite3.Row
            connection.execute("PRAGMA foreign_keys = OFF")
            connection.commit()
            os.chmod(path, 0o600)
        except StoreError:
            raise
        except Exception as exc:
            if self.logger is not None:
                self.logger.log_message(
                    "database open failed",
                    level="ERROR",
                    component="Store",
                )
            raise StoreError(
                "Cannot open the sqlite3 file {0}.".format(path),
                "check that the folder is writable, or set PYRBAC_SQLITE_FILE, then py-rbac db-init",
                logger=self.logger,
            ) from exc
        return connection

    def _connect_mysql(self):
        try:
            import pymysql
            from pymysql.cursors import DictCursor
        except ImportError:
            raise StoreError(
                "PyMySQL is required.",
                'python -m pip install "PyMySQL>=1.1.0"',
                logger=self.logger,
            )
        try:
            cfg = self.settings.database()
        except Exception as exc:
            nxt = getattr(exc, "nxt", "check PYRBAC_DB_PASSWORD")
            raise StoreError(str(exc), nxt, logger=self.logger)
        try:
            return pymysql.connect(
                host=cfg["host"],
                port=cfg["port"],
                user=cfg["user"],
                password=cfg["password"],
                database=cfg["database"],
                charset="utf8mb4",
                cursorclass=DictCursor,
                autocommit=False,
            )
        except Exception as exc:
            if self.logger is not None:
                self.logger.log_message(
                    "database open failed",
                    level="ERROR",
                    component="Store",
                )
            raise StoreError(
                "Cannot open the MySQL database {0}.".format(cfg["database"]),
                "check PYRBAC_DB_HOST, PYRBAC_DB_USER, and PYRBAC_DB_PASSWORD, then py-rbac db-init",
                logger=self.logger,
            ) from exc

    def _sql(self, sql):
        """sqlite3 uses ?, datetime('now','localtime'), and instr. mysql stays as written."""
        if self.settings.engine() != "sqlite3":
            return sql
        text = sql.replace("%s", "?")
        text = re.sub(r"\bNOW\(\)", "datetime('now','localtime')", text, flags=re.IGNORECASE)
        text = re.sub(
            r"FIND_IN_SET\(\s*\?\s*,\s*([A-Za-z0-9_.]+)\s*\)",
            r"(instr(',' || \1 || ',', ',' || ? || ',') > 0)",
            text,
            flags=re.IGNORECASE,
        )
        return text

    def _dicts(self, rows):
        found = []
        for row in rows:
            if isinstance(row, dict):
                found.append(row)
            else:
                found.append(dict(row))
        return found

    def _execute(self, connection, sql, args, fetch):
        cursor = connection.cursor()
        try:
            cursor.execute(self._sql(sql), args)
            rows = cursor.fetchall() if fetch else []
            return {
                "rows": self._dicts(rows),
                "lastrowid": cursor.lastrowid,
                "rowcount": cursor.rowcount,
            }
        finally:
            cursor.close()

    def _run(self, sql, args, fetch):
        connection = self.connect()
        try:
            done = self._execute(connection, sql, args, fetch)
            connection.commit()
            return done
        except StoreError:
            raise
        except Exception as exc:
            try:
                connection.rollback()
            except Exception:
                pass
            if self.logger is not None:
                self.logger.log_message(
                    "database statement failed",
                    level="ERROR",
                    component="Store",
                )
            raise StoreError(
                "The database statement failed.",
                "py-rbac db-init",
                logger=self.logger,
            ) from exc
        finally:
            connection.close()

    def query_all(self, sql, args=()):
        """General Purpose: Every row from one parameterized statement."""
        return self._run(sql, args, True)["rows"]

    def query_one(self, sql, args=()):
        """General Purpose: The first row, or None."""
        rows = self.query_all(sql, args)
        if not rows:
            return None
        return rows[0]

    def execute(self, sql, args=()):
        """General Purpose: One write statement. Returns lastrowid and rowcount."""
        return self._run(sql, args, False)

    def transaction(self, work):
        """General Purpose: Several statements. work(run) sees one connection. Commit is once."""
        connection = self.connect()

        def run(sql, args=()):
            return self._execute(connection, sql, args, True)

        try:
            result = work(run)
            connection.commit()
            return result
        except StoreError:
            try:
                connection.rollback()
            except Exception:
                pass
            raise
        except Exception as exc:
            try:
                connection.rollback()
            except Exception:
                pass
            if self.logger is not None:
                self.logger.log_message(
                    "database transaction failed",
                    level="ERROR",
                    component="Store",
                )
            raise StoreError(
                "The database transaction failed.",
                "py-rbac db-init",
                logger=self.logger,
            ) from exc
        finally:
            connection.close()
