# =============================================================================
# Web interface for the same RBAC actions as the text menu.
# requirement-web-interface — loopback pages, cookie session, permission checks.
# requirement-python-oop — class WebApp. The standard-library handler is created
# inside make_server because that library constructs the handler itself.
# =============================================================================
from __future__ import annotations

import html
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

from .settings import SettingsError


class WebApp:
    """Small HTTP site. Every mutating page calls Access or Catalog."""

    COOKIE = "pyrbac_session"

    def __init__(self, settings, access, catalog, report, logger=None, prepare=None):
        self.logger = logger
        if logger is not None:
            logger.log_message("instantiated", component="WebApp")
        self.settings = settings
        self.access = access
        self.catalog = catalog
        self.report = report
        self.prepare = prepare
        self.httpd = None
        self.thread = None

    def url(self) -> str:
        if self.httpd is None:
            host, port = self.settings.bind()
        else:
            host, port = self.httpd.server_address[0], self.httpd.server_address[1]
        return "http://{0}:{1}/".format(host, port)

    def make_server(self):
        """General Purpose: A loopback server unless remote bind was explicitly allowed."""
        host, port = self.settings.bind()
        outer = self

        class Handler(BaseHTTPRequestHandler):
            """Constructed by ThreadingHTTPServer. WebApp owns the actions."""

            def do_GET(self):
                outer.get(self)

            def do_POST(self):
                outer.post(self)

            def log_message(self, fmt, *args):
                if outer.logger is not None:
                    outer.logger.log_message(
                        "http {0}".format(fmt % args),
                        component="WebApp",
                    )

        self.httpd = ThreadingHTTPServer((host, port), Handler)
        return self.httpd

    def serve_foreground(self):
        """General Purpose: Serve until the process is interrupted."""
        server = self.make_server()
        if self.logger is not None:
            self.logger.log_message(
                "listening {0}".format(self.url()),
                component="WebApp",
            )
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass
        finally:
            self.stop()

    def start_background(self) -> str:
        """General Purpose: Serve on a daemon thread and return the URL."""
        if self.thread is not None and self.thread.is_alive():
            return self.url()
        self.make_server()
        self.thread = threading.Thread(target=self.httpd.serve_forever, daemon=True)
        self.thread.start()
        if self.logger is not None:
            self.logger.log_message(
                "listening {0}".format(self.url()),
                component="WebApp",
            )
        return self.url()

    def stop(self):
        """General Purpose: Stop the server if this object started one."""
        server = self.httpd
        if server is not None:
            try:
                server.shutdown()
            except Exception:
                pass
            try:
                server.server_close()
            except Exception:
                pass
        self.httpd = None
        self.thread = None

    def get(self, handler):
        try:
            path = urlparse(handler.path).path
            user = self._user(handler)
            if path == "/login":
                self._send(handler, 200, self.login_page(""))
                return
            if user is None:
                self._redirect(handler, "/login")
                return
            query = parse_qs(urlparse(handler.path).query)
            error = self._first(query, "error")
            body = self._route_get(path, user, error)
            if body is None:
                self._send(handler, 404, self.page(user, "Not found", "<p>That page is not on this site.</p>"))
                return
            self._send(handler, 200, body)
        except Exception as exc:
            self._fail(handler, exc)

    def post(self, handler):
        try:
            path = urlparse(handler.path).path
            form = self._form(handler)
            if path == "/login":
                self._post_login(handler, form)
                return
            user = self._user(handler)
            if user is None:
                self._redirect(handler, "/login")
                return
            if not self._csrf_ok(user, form):
                self._send(handler, 400, self.page(user, "Rejected", "<p>The form token did not match this session.</p>"))
                return
            self._route_post(handler, path, user, form)
        except Exception as exc:
            self._fail(handler, exc)

    def _route_get(self, path, user, error):
        if path == "/":
            return self.home_page(user, error)
        if path == "/users":
            self.access.require(user, "system:user:list")
            return self.users_page(user, error)
        if path == "/roles":
            self.access.require(user, "system:role:list")
            return self.roles_page(user, error)
        if path == "/menus":
            self.access.require(user, "system:menu:list")
            return self.menus_page(user, error)
        if path == "/depts":
            self.access.require(user, "system:dept:list")
            return self.depts_page(user, error)
        if path == "/posts":
            self.access.require(user, "system:post:list")
            return self.posts_page(user, error)
        if path == "/configs":
            self.catalog.access.require(user, "system:config:list")
            return self.configs_page(user, error)
        if path == "/dicts":
            self.catalog.access.require(user, "system:dict:list")
            return self.dicts_page(user, error)
        if path == "/notices":
            self.catalog.access.require(user, "system:notice:list")
            return self.notices_page(user, error)
        if path == "/oper-log":
            self.catalog.access.require(user, "monitor:operlog:list")
            return self.simple_page(user, "Operation log", self.report.oper_log(self.catalog.oper_log(user)), error)
        if path == "/login-log":
            self.catalog.access.require(user, "monitor:logininfor:list")
            return self.simple_page(user, "Login log", self.report.login_log(self.catalog.login_log(user)), error)
        if path == "/online":
            self.catalog.access.require(user, "monitor:online:list")
            return self.online_page(user, error)
        if path == "/database":
            self.access.require(user, "system:config:edit")
            return self.database_page(user, error)
        return None

    def _route_post(self, handler, path, user, form):
        if path == "/logout":
            self.access.logout(user.get("token"))
            self._redirect(handler, "/login", clear=True)
            return
        if path == "/users/add":
            self.access.user_add(
                user,
                self._first(form, "user_name"),
                self._first(form, "nick_name"),
                int(self._first(form, "dept_id") or "0"),
                int(self._first(form, "role_id") or "0"),
                self._first(form, "password"),
            )
            self._redirect(handler, "/users")
            return
        if path == "/users/edit":
            self.access.user_edit(
                user,
                int(self._first(form, "user_id") or "0"),
                nick_name=self._first(form, "nick_name") or None,
                dept_id=self._optional_int(form, "dept_id"),
                status=self._first(form, "status") or None,
            )
            self._redirect(handler, "/users")
            return
        if path == "/users/remove":
            self.access.user_remove(user, int(self._first(form, "user_id") or "0"))
            self._redirect(handler, "/users")
            return
        if path == "/users/reset":
            self.access.user_reset_password(
                user,
                int(self._first(form, "user_id") or "0"),
                self._first(form, "password"),
            )
            self._redirect(handler, "/users")
            return
        if path == "/roles/grant":
            menus = [int(item) for item in form.get("menu_id", []) if item]
            depts = [int(item) for item in form.get("dept_id", []) if item]
            scope = self._first(form, "data_scope") or None
            self.access.role_grant(
                user,
                int(self._first(form, "role_id") or "0"),
                menus,
                data_scope=scope,
                dept_ids=depts,
            )
            self._redirect(handler, "/roles")
            return
        if path == "/menus/add":
            self.access.menu_add(
                user,
                self._first(form, "menu_name"),
                int(self._first(form, "parent_id") or "0"),
                self._first(form, "menu_type") or "F",
                self._first(form, "perms"),
                self._first(form, "path"),
            )
            self._redirect(handler, "/menus")
            return
        if path == "/depts/add":
            self.access.dept_add(
                user,
                self._first(form, "dept_name"),
                int(self._first(form, "parent_id") or "0"),
            )
            self._redirect(handler, "/depts")
            return
        if path == "/posts/add":
            self.access.post_add(
                user,
                self._first(form, "post_code"),
                self._first(form, "post_name"),
            )
            self._redirect(handler, "/posts")
            return
        if path == "/configs/set":
            self.catalog.config_set(
                user,
                self._first(form, "config_key"),
                self._first(form, "config_value"),
            )
            self._redirect(handler, "/configs")
            return
        if path == "/dicts/add":
            self.catalog.dict_add(
                user,
                self._first(form, "dict_type"),
                self._first(form, "dict_label"),
                self._first(form, "dict_value"),
            )
            self._redirect(handler, "/dicts")
            return
        if path == "/notices/add":
            self.catalog.notice_add(
                user,
                self._first(form, "title"),
                self._first(form, "notice_type") or "1",
                self._first(form, "content"),
            )
            self._redirect(handler, "/notices")
            return
        if path == "/online/logout":
            self.catalog.online_logout(user, self._first(form, "token"))
            self._redirect(handler, "/online")
            return
        if path == "/database":
            self.access.require(user, "system:config:edit")
            before = self.settings.engine()
            self._apply_posted_engine(handler, form, allow_remote=True)
            if self.settings.engine() != before:
                self._redirect(handler, "/login", clear=True)
                return
            self._redirect(handler, "/database")
            return
        self._send(handler, 404, self.page(user, "Not found", "<p>That action is not on this site.</p>"))

    def _loopback(self, handler) -> bool:
        host = ""
        if getattr(handler, "client_address", None):
            host = handler.client_address[0]
        return host in ("127.0.0.1", "::1", "localhost")

    def _apply_posted_engine(self, handler, form, allow_remote=False):
        """Store a posted sqlite3 or mysql choice. A remote login cannot switch engines."""
        posted = self._first(form, "engine").strip().lower()
        if posted == "":
            return
        if posted in ("1", "sqlite3"):
            wanted = "sqlite3"
        elif posted in ("2", "mysql"):
            wanted = "mysql"
        else:
            raise SettingsError(
                "The database engine must be sqlite3 or mysql.",
                "choose sqlite3 or mysql",
                logger=self.logger,
            )
        if wanted == self.settings.engine():
            return
        if not allow_remote and not self._loopback(handler):
            raise SettingsError(
                "The database engine can be changed from this computer.",
                "open the site on 127.0.0.1, or run py-rbac db-engine",
                logger=self.logger,
            )
        self.settings.apply_choice(wanted)

    def _post_login(self, handler, form):
        self._apply_posted_engine(handler, form)
        if self.prepare is not None:
            self.prepare()
        user = self.access.login(
            self._first(form, "user_name"),
            self._first(form, "password"),
            ipaddr=handler.client_address[0] if handler.client_address else "",
            surface="web",
            user_agent=handler.headers.get("User-Agent", ""),
        )
        self._redirect(handler, "/", cookie=user["token"])

    def _fail(self, handler, exc):
        message = str(exc)
        nxt = getattr(exc, "nxt", "")
        body = "<p class=\"error\">{0}</p>".format(self._e(message))
        if nxt:
            body += "<p>{0}</p>".format(self._e(nxt))
        code = 403 if message.startswith("Missing permission") else 400
        try:
            user = self._user(handler)
        except Exception:
            user = None
        self._send(handler, code, self.page(user, "Not completed", body))

    def _user(self, handler):
        token = self._cookie(handler).get(self.COOKIE, "")
        if not token:
            return None
        return self.access.actor_from_token(token)

    def _cookie(self, handler):
        raw = handler.headers.get("Cookie", "")
        found = {}
        for part in raw.split(";"):
            if "=" not in part:
                continue
            key, value = part.split("=", 1)
            found[key.strip()] = value.strip()
        return found

    def _form(self, handler):
        length = int(handler.headers.get("Content-Length", "0") or "0")
        if length < 0 or length > 100000:
            raise ValueError("The form is too large.")
        raw = handler.rfile.read(length).decode("utf-8", "replace")
        return parse_qs(raw, keep_blank_values=True)

    def _first(self, form, name):
        values = form.get(name) or [""]
        return values[0]

    def _optional_int(self, form, name):
        raw = self._first(form, name)
        if raw == "":
            return None
        return int(raw)

    def _csrf_ok(self, user, form):
        return self._first(form, "csrf") != "" and self._first(form, "csrf") == user.get("csrf")

    def _hidden(self, user):
        return '<input type="hidden" name="csrf" value="{0}">'.format(self._e(user.get("csrf") or ""))

    def _e(self, value) -> str:
        return html.escape("" if value is None else str(value), quote=True)

    def _send(self, handler, code, body, extra=None):
        data = body.encode("utf-8")
        handler.send_response(code)
        handler.send_header("Content-Type", "text/html; charset=utf-8")
        handler.send_header("Content-Length", str(len(data)))
        handler.send_header("X-Content-Type-Options", "nosniff")
        handler.send_header("Referrer-Policy", "no-referrer")
        handler.send_header(
            "Content-Security-Policy",
            "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'",
        )
        if extra:
            for key, value in extra:
                handler.send_header(key, value)
        handler.end_headers()
        handler.wfile.write(data)

    def _redirect(self, handler, location, cookie=None, clear=False):
        handler.send_response(303)
        handler.send_header("Location", location)
        handler.send_header("Content-Length", "0")
        if cookie is not None:
            handler.send_header(
                "Set-Cookie",
                "{0}={1}; HttpOnly; SameSite=Lax; Path=/".format(self.COOKIE, cookie),
            )
        if clear:
            handler.send_header(
                "Set-Cookie",
                "{0}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0".format(self.COOKIE),
            )
        handler.end_headers()

    def nav(self, user) -> str:
        if user is None:
            return "<nav><a href=\"/login\">Sign in</a></nav>"
        links = [("/", "Home")]
        pairs = (
            ("/users", "Users", "system:user:list"),
            ("/roles", "Roles", "system:role:list"),
            ("/menus", "Menus", "system:menu:list"),
            ("/depts", "Departments", "system:dept:list"),
            ("/posts", "Posts", "system:post:list"),
            ("/configs", "Parameters", "system:config:list"),
            ("/dicts", "Dictionaries", "system:dict:list"),
            ("/notices", "Notices", "system:notice:list"),
            ("/oper-log", "Operation log", "monitor:operlog:list"),
            ("/login-log", "Login log", "monitor:logininfor:list"),
            ("/online", "Online", "monitor:online:list"),
            ("/database", "Database", "system:config:edit"),
        )
        for href, label, perm in pairs:
            if self.access.allows(user["user_id"], perm):
                links.append((href, label))
        parts = ["<a href=\"{0}\">{1}</a>".format(href, self._e(label)) for href, label in links]
        parts.append(
            "<form method=\"post\" action=\"/logout\" style=\"display:inline\">{0}"
            "<button type=\"submit\">Sign out</button></form>".format(self._hidden(user))
        )
        return "<nav>{0}</nav>".format(" ".join(parts))

    def page(self, user, title, body) -> str:
        note = ""
        if user is None:
            note = ""
        return (
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">"
            "<title>{title}</title><style>"
            "body{{font-family:sans-serif;margin:1.5rem;max-width:60rem}}"
            "nav a{{margin-right:.7rem}} table{{border-collapse:collapse;margin:1rem 0}}"
            "td,th{{border:1px solid #ccc;padding:.25rem .45rem;text-align:left}}"
            ".error{{color:#8a1f1f}} form.inline{{display:inline}}"
            "</style></head><body><h1>{title}</h1>{nav}{note}{body}</body></html>"
        ).format(
            title=self._e(title),
            nav=self.nav(user),
            note=note,
            body=body,
        )

    def _error_line(self, error) -> str:
        if not error:
            return ""
        return "<p class=\"error\">{0}</p>".format(self._e(error))

    def engine_fields(self) -> str:
        """The two engine choices. sqlite3 names the file, including the default file."""
        current = self.settings.engine()
        path = self.settings.sqlite_path()
        mark = " (default file)" if self.settings.get("PYRBAC_SQLITE_FILE") == "" else ""
        name = self.settings.get("PYRBAC_DB_NAME")

        def radio(value, label):
            checked = " checked" if current == value else ""
            return (
                '<label><input type="radio" name="engine" value="{value}"{checked}> {label}</label>'
            ).format(value=value, checked=checked, label=self._e(label))

        return (
            "<fieldset><legend>Database</legend>"
            "<p>{sqlite}</p><p>{mysql}</p></fieldset>"
        ).format(
            sqlite=radio("sqlite3", "sqlite3 {0}{1}".format(path, mark)),
            mysql=radio("mysql", "mysql {0}".format(name)),
        )

    def login_page(self, error) -> str:
        body = (
            "{error}<form method=\"post\" action=\"/login\">{engine}"
            "<p><label>Account <input name=\"user_name\" autocomplete=\"username\"></label></p>"
            "<p><label>Password <input name=\"password\" type=\"password\" autocomplete=\"current-password\"></label></p>"
            "<p><button type=\"submit\">Sign in</button></p></form>"
            "<p>Seeded accounts and initial passwords are in the product README. "
            "Change them after the first sign-in. This site listens on this computer only "
            "unless remote bind was explicitly allowed. "
            "The default database is the sqlite3 file. mysql needs PYRBAC_DB_PASSWORD.</p>"
        ).format(error=self._error_line(error), engine=self.engine_fields())
        return self.page(None, "Sign in", body)

    def database_page(self, user, error) -> str:
        body = (
            "{error}<p>{sentence} The next sign-in uses the saved engine. "
            "Changing it signs this browser out, because the session row stays in the previous database.</p>"
            "<form method=\"post\" action=\"/database\">{engine}{csrf}"
            "<p><button type=\"submit\">Save</button></p></form>"
        ).format(
            error=self._error_line(error),
            sentence=self._e(self.settings.engine_sentence()),
            engine=self.engine_fields(),
            csrf=self._hidden(user),
        )
        return self.page(user, "Database", body)

    def home_page(self, user, error) -> str:
        body = "{error}<p>Signed in as <strong>{name}</strong>.</p><pre>{who}</pre>".format(
            error=self._error_line(error),
            name=self._e(user["user_name"]),
            who=self._e(self.access.describe(user)),
        )
        return self.page(user, "Home", body)

    def _rows_html(self, columns, rows) -> str:
        head = "".join("<th>{0}</th>".format(self._e(name)) for name in columns)
        body = []
        for row in rows:
            cells = []
            for name in columns:
                cells.append("<td>{0}</td>".format(self._e(row.get(name))))
            body.append("<tr>{0}</tr>".format("".join(cells)))
        if not body:
            body.append("<tr><td colspan=\"{0}\">(none)</td></tr>".format(len(columns)))
        return "<table><tr>{0}</tr>{1}</table>".format(head, "".join(body))

    def users_page(self, user, error) -> str:
        rows = self.access.user_list(user)
        parts = [self._error_line(error), self._rows_html(
            ("user_id", "user_name", "nick_name", "dept_id", "dept_name", "status"),
            rows,
        )]
        if self.access.allows(user["user_id"], "system:user:add"):
            parts.append(
                "<h2>Add a user</h2><form method=\"post\" action=\"/users/add\">{csrf}"
                "<p>Account <input name=\"user_name\"></p>"
                "<p>Name <input name=\"nick_name\"></p>"
                "<p>Department id <input name=\"dept_id\"></p>"
                "<p>Role id <input name=\"role_id\"></p>"
                "<p>Password <input name=\"password\" type=\"password\"></p>"
                "<p><button type=\"submit\">Add</button></p></form>".format(csrf=self._hidden(user))
            )
        if self.access.allows(user["user_id"], "system:user:edit"):
            parts.append(
                "<h2>Edit a user</h2><form method=\"post\" action=\"/users/edit\">{csrf}"
                "<p>User id <input name=\"user_id\"></p>"
                "<p>Name <input name=\"nick_name\"></p>"
                "<p>Department id <input name=\"dept_id\"></p>"
                "<p>Status <input name=\"status\" placeholder=\"0 or 1\"></p>"
                "<p><button type=\"submit\">Save</button></p></form>".format(csrf=self._hidden(user))
            )
        if self.access.allows(user["user_id"], "system:user:resetPwd"):
            parts.append(
                "<h2>Reset a password</h2><form method=\"post\" action=\"/users/reset\">{csrf}"
                "<p>User id <input name=\"user_id\"></p>"
                "<p>Password <input name=\"password\" type=\"password\"></p>"
                "<p><button type=\"submit\">Reset</button></p></form>".format(csrf=self._hidden(user))
            )
        if self.access.allows(user["user_id"], "system:user:remove"):
            parts.append(
                "<h2>Remove a user</h2><form method=\"post\" action=\"/users/remove\">{csrf}"
                "<p>User id <input name=\"user_id\"></p>"
                "<p><button type=\"submit\">Remove</button></p></form>".format(csrf=self._hidden(user))
            )
        return self.page(user, "Users", "".join(parts))

    def roles_page(self, user, error) -> str:
        rows = self.access.role_list(user)
        menus = []
        if self.access.allows(user["user_id"], "system:menu:list") or self.access.allows(user["user_id"], "system:role:edit"):
            try:
                menus = self.access.menu_list(user) if self.access.allows(user["user_id"], "system:menu:list") else []
            except Exception:
                menus = self.store_menus_for_grant(user)
        parts = [self._error_line(error), self._rows_html(
            ("role_id", "role_name", "role_key", "data_scope", "status"),
            rows,
        )]
        if self.access.allows(user["user_id"], "system:role:edit"):
            boxes = []
            for menu in self.access.menu_list(user) if self.access.is_admin(user["user_id"]) or self.access.allows(user["user_id"], "system:menu:list") else self._menus_unscoped():
                boxes.append(
                    "<label><input type=\"checkbox\" name=\"menu_id\" value=\"{0}\"> {1} {2}</label><br>".format(
                        self._e(menu["menu_id"]),
                        self._e(menu["menu_name"]),
                        self._e(menu.get("perms") or ""),
                    )
                )
            parts.append(
                "<h2>Grant menus</h2><form method=\"post\" action=\"/roles/grant\">{csrf}"
                "<p>Role id <input name=\"role_id\"></p>"
                "<p>Data scope <input name=\"data_scope\" placeholder=\"1, 2, 3, or 4\"></p>"
                "<p>Custom department ids <input name=\"dept_id\"></p>"
                "<div>{boxes}</div>"
                "<p><button type=\"submit\">Grant</button></p></form>".format(
                    csrf=self._hidden(user),
                    boxes="".join(boxes),
                )
            )
        return self.page(user, "Roles", "".join(parts))

    def _menus_unscoped(self):
        """Menu ids for the grant form when the actor may edit roles but not list menus.

        The grant still rejects unknown ids. The form needs the seeded ids.
        """
        return self.access.store.query_all(
            """
            SELECT menu_id, menu_name, perms FROM sys_menu
            WHERE status = '0' ORDER BY parent_id, order_num, menu_id
            """
        )

    def menus_page(self, user, error) -> str:
        parts = [self._error_line(error), self._rows_html(
            ("menu_id", "parent_id", "menu_name", "menu_type", "perms"),
            self.access.menu_list(user),
        )]
        if self.access.allows(user["user_id"], "system:menu:add"):
            parts.append(
                "<h2>Add a menu</h2><form method=\"post\" action=\"/menus/add\">{csrf}"
                "<p>Name <input name=\"menu_name\"></p>"
                "<p>Parent id <input name=\"parent_id\" value=\"0\"></p>"
                "<p>Type <input name=\"menu_type\" placeholder=\"M, C, or F\"></p>"
                "<p>Permission <input name=\"perms\"></p>"
                "<p>Path <input name=\"path\"></p>"
                "<p><button type=\"submit\">Add</button></p></form>".format(csrf=self._hidden(user))
            )
        return self.page(user, "Menus", "".join(parts))

    def depts_page(self, user, error) -> str:
        parts = [self._error_line(error), self._rows_html(
            ("dept_id", "parent_id", "dept_name", "ancestors", "status"),
            self.access.dept_list(user),
        )]
        if self.access.allows(user["user_id"], "system:dept:add"):
            parts.append(
                "<h2>Add a department</h2><form method=\"post\" action=\"/depts/add\">{csrf}"
                "<p>Name <input name=\"dept_name\"></p>"
                "<p>Parent id <input name=\"parent_id\" value=\"0\"></p>"
                "<p><button type=\"submit\">Add</button></p></form>".format(csrf=self._hidden(user))
            )
        return self.page(user, "Departments", "".join(parts))

    def posts_page(self, user, error) -> str:
        parts = [self._error_line(error), self._rows_html(
            ("post_id", "post_code", "post_name", "post_sort", "status"),
            self.access.post_list(user),
        )]
        if self.access.allows(user["user_id"], "system:post:add"):
            parts.append(
                "<h2>Add a post</h2><form method=\"post\" action=\"/posts/add\">{csrf}"
                "<p>Code <input name=\"post_code\"></p>"
                "<p>Name <input name=\"post_name\"></p>"
                "<p><button type=\"submit\">Add</button></p></form>".format(csrf=self._hidden(user))
            )
        return self.page(user, "Posts", "".join(parts))

    def configs_page(self, user, error) -> str:
        parts = [self._error_line(error), self._rows_html(
            ("config_key", "config_name", "config_value", "config_type"),
            self.catalog.config_list(user),
        )]
        if self.access.allows(user["user_id"], "system:config:edit"):
            parts.append(
                "<h2>Change a parameter</h2><form method=\"post\" action=\"/configs/set\">{csrf}"
                "<p>Key <input name=\"config_key\"></p>"
                "<p>Value <input name=\"config_value\"></p>"
                "<p><button type=\"submit\">Save</button></p></form>".format(csrf=self._hidden(user))
            )
        return self.page(user, "Parameters", "".join(parts))

    def dicts_page(self, user, error) -> str:
        parts = [self._error_line(error), self._rows_html(
            ("dict_type", "dict_label", "dict_value", "status"),
            self.catalog.dict_list(user),
        )]
        if self.access.allows(user["user_id"], "system:dict:add"):
            parts.append(
                "<h2>Add a dictionary value</h2><form method=\"post\" action=\"/dicts/add\">{csrf}"
                "<p>Type <input name=\"dict_type\"></p>"
                "<p>Label <input name=\"dict_label\"></p>"
                "<p>Value <input name=\"dict_value\"></p>"
                "<p><button type=\"submit\">Add</button></p></form>".format(csrf=self._hidden(user))
            )
        return self.page(user, "Dictionaries", "".join(parts))

    def notices_page(self, user, error) -> str:
        parts = [self._error_line(error), self._rows_html(
            ("notice_id", "notice_title", "notice_type", "status"),
            self.catalog.notice_list(user),
        )]
        if self.access.allows(user["user_id"], "system:notice:add"):
            parts.append(
                "<h2>Publish a notice</h2><form method=\"post\" action=\"/notices/add\">{csrf}"
                "<p>Title <input name=\"title\"></p>"
                "<p>Type <input name=\"notice_type\" placeholder=\"1 or 2\"></p>"
                "<p>Content <input name=\"content\"></p>"
                "<p><button type=\"submit\">Publish</button></p></form>".format(csrf=self._hidden(user))
            )
        return self.page(user, "Notices", "".join(parts))

    def simple_page(self, user, title, text, error) -> str:
        body = "{error}<pre>{text}</pre>".format(
            error=self._error_line(error),
            text=self._e(text),
        )
        return self.page(user, title, body)

    def online_page(self, user, error) -> str:
        rows = self.catalog.online_list(user)
        parts = [self._error_line(error)]
        for row in rows:
            token = row.get("token") or ""
            parts.append(
                "<p>{name} {surface} {prefix}… ".format(
                    name=self._e(row.get("user_name")),
                    surface=self._e(row.get("surface")),
                    prefix=self._e(str(token)[:8]),
                )
            )
            if self.access.allows(user["user_id"], "monitor:online:forceLogout"):
                parts.append(
                    "<form class=\"inline\" method=\"post\" action=\"/online/logout\">{csrf}"
                    "<input type=\"hidden\" name=\"token\" value=\"{token}\">"
                    "<button type=\"submit\">Force logout</button></form>".format(
                        csrf=self._hidden(user),
                        token=self._e(token),
                    )
                )
            parts.append("</p>")
        if len(rows) == 0:
            parts.append("<p>(none)</p>")
        return self.page(user, "Online", "".join(parts))

    def store_menus_for_grant(self, user):
        return self._menus_unscoped()
