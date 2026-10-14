# =============================================================================
# Tables and the first seed for PyRbac.
# requirement-database — the same columns on sqlite3 and on MySQL.
# requirement-rbac — seed accounts, roles, menus, and permission strings.
# requirement-python-oop — class Schema.
# =============================================================================
from __future__ import annotations

import re

from .password_hash import PasswordHash


class Schema:
    """Create the RBAC tables and seed them once when sys_user is empty.

    This product writes its own SQL. The password column is VARCHAR(255) because
    the hash is PBKDF2. Online sessions live in sys_session.
    """

    INITIAL_ADMIN_PASSWORD = "admin123"
    INITIAL_OPERATOR_PASSWORD = "operator123"
    INITIAL_LEAD_PASSWORD = "lead123"
    INITIAL_CLERK_PASSWORD = "clerk123"

    STATEMENTS = (
        """
        CREATE TABLE IF NOT EXISTS sys_dept (
          dept_id BIGINT NOT NULL AUTO_INCREMENT,
          parent_id BIGINT NOT NULL DEFAULT 0,
          ancestors VARCHAR(50) NOT NULL DEFAULT '',
          dept_name VARCHAR(30) NOT NULL DEFAULT '',
          order_num INT NOT NULL DEFAULT 0,
          leader VARCHAR(20) DEFAULT NULL,
          phone VARCHAR(11) DEFAULT NULL,
          email VARCHAR(50) DEFAULT NULL,
          status CHAR(1) NOT NULL DEFAULT '0',
          del_flag CHAR(1) NOT NULL DEFAULT '0',
          create_by VARCHAR(64) NOT NULL DEFAULT '',
          create_time DATETIME DEFAULT NULL,
          update_by VARCHAR(64) NOT NULL DEFAULT '',
          update_time DATETIME DEFAULT NULL,
          PRIMARY KEY (dept_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_user (
          user_id BIGINT NOT NULL AUTO_INCREMENT,
          dept_id BIGINT DEFAULT NULL,
          user_name VARCHAR(30) NOT NULL,
          nick_name VARCHAR(30) NOT NULL,
          user_type VARCHAR(2) NOT NULL DEFAULT '00',
          email VARCHAR(50) NOT NULL DEFAULT '',
          phonenumber VARCHAR(11) NOT NULL DEFAULT '',
          sex CHAR(1) NOT NULL DEFAULT '2',
          avatar VARCHAR(100) NOT NULL DEFAULT '',
          password VARCHAR(255) NOT NULL DEFAULT '',
          status CHAR(1) NOT NULL DEFAULT '0',
          del_flag CHAR(1) NOT NULL DEFAULT '0',
          login_ip VARCHAR(128) NOT NULL DEFAULT '',
          login_date DATETIME DEFAULT NULL,
          pwd_update_date DATETIME DEFAULT NULL,
          create_by VARCHAR(64) NOT NULL DEFAULT '',
          create_time DATETIME DEFAULT NULL,
          update_by VARCHAR(64) NOT NULL DEFAULT '',
          update_time DATETIME DEFAULT NULL,
          remark VARCHAR(500) DEFAULT NULL,
          PRIMARY KEY (user_id),
          UNIQUE KEY uk_sys_user_name (user_name)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_post (
          post_id BIGINT NOT NULL AUTO_INCREMENT,
          post_code VARCHAR(64) NOT NULL,
          post_name VARCHAR(50) NOT NULL,
          post_sort INT NOT NULL,
          status CHAR(1) NOT NULL DEFAULT '0',
          create_by VARCHAR(64) NOT NULL DEFAULT '',
          create_time DATETIME DEFAULT NULL,
          update_by VARCHAR(64) NOT NULL DEFAULT '',
          update_time DATETIME DEFAULT NULL,
          remark VARCHAR(500) DEFAULT NULL,
          PRIMARY KEY (post_id),
          UNIQUE KEY uk_sys_post_code (post_code)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_role (
          role_id BIGINT NOT NULL AUTO_INCREMENT,
          role_name VARCHAR(30) NOT NULL,
          role_key VARCHAR(100) NOT NULL,
          role_sort INT NOT NULL,
          data_scope CHAR(1) NOT NULL DEFAULT '1',
          menu_check_strictly TINYINT NOT NULL DEFAULT 1,
          dept_check_strictly TINYINT NOT NULL DEFAULT 1,
          status CHAR(1) NOT NULL,
          del_flag CHAR(1) NOT NULL DEFAULT '0',
          create_by VARCHAR(64) NOT NULL DEFAULT '',
          create_time DATETIME DEFAULT NULL,
          update_by VARCHAR(64) NOT NULL DEFAULT '',
          update_time DATETIME DEFAULT NULL,
          remark VARCHAR(500) DEFAULT NULL,
          PRIMARY KEY (role_id),
          UNIQUE KEY uk_sys_role_key (role_key)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_menu (
          menu_id BIGINT NOT NULL AUTO_INCREMENT,
          menu_name VARCHAR(50) NOT NULL,
          parent_id BIGINT NOT NULL DEFAULT 0,
          order_num INT NOT NULL DEFAULT 0,
          path VARCHAR(200) NOT NULL DEFAULT '',
          component VARCHAR(255) DEFAULT NULL,
          query VARCHAR(255) DEFAULT NULL,
          route_name VARCHAR(50) NOT NULL DEFAULT '',
          is_frame INT NOT NULL DEFAULT 1,
          is_cache INT NOT NULL DEFAULT 0,
          menu_type CHAR(1) NOT NULL DEFAULT '',
          visible CHAR(1) NOT NULL DEFAULT '0',
          status CHAR(1) NOT NULL DEFAULT '0',
          perms VARCHAR(100) DEFAULT NULL,
          icon VARCHAR(100) NOT NULL DEFAULT '#',
          create_by VARCHAR(64) NOT NULL DEFAULT '',
          create_time DATETIME DEFAULT NULL,
          update_by VARCHAR(64) NOT NULL DEFAULT '',
          update_time DATETIME DEFAULT NULL,
          remark VARCHAR(500) NOT NULL DEFAULT '',
          PRIMARY KEY (menu_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_user_role (
          user_id BIGINT NOT NULL,
          role_id BIGINT NOT NULL,
          PRIMARY KEY (user_id, role_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_user_post (
          user_id BIGINT NOT NULL,
          post_id BIGINT NOT NULL,
          PRIMARY KEY (user_id, post_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_role_menu (
          role_id BIGINT NOT NULL,
          menu_id BIGINT NOT NULL,
          PRIMARY KEY (role_id, menu_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_role_dept (
          role_id BIGINT NOT NULL,
          dept_id BIGINT NOT NULL,
          PRIMARY KEY (role_id, dept_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_dict_type (
          dict_id BIGINT NOT NULL AUTO_INCREMENT,
          dict_name VARCHAR(100) NOT NULL DEFAULT '',
          dict_type VARCHAR(100) NOT NULL DEFAULT '',
          status CHAR(1) NOT NULL DEFAULT '0',
          create_by VARCHAR(64) NOT NULL DEFAULT '',
          create_time DATETIME DEFAULT NULL,
          update_by VARCHAR(64) NOT NULL DEFAULT '',
          update_time DATETIME DEFAULT NULL,
          remark VARCHAR(500) DEFAULT NULL,
          PRIMARY KEY (dict_id),
          UNIQUE KEY uk_sys_dict_type (dict_type)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_dict_data (
          dict_code BIGINT NOT NULL AUTO_INCREMENT,
          dict_sort INT NOT NULL DEFAULT 0,
          dict_label VARCHAR(100) NOT NULL DEFAULT '',
          dict_value VARCHAR(100) NOT NULL DEFAULT '',
          dict_type VARCHAR(100) NOT NULL DEFAULT '',
          css_class VARCHAR(100) DEFAULT NULL,
          list_class VARCHAR(100) DEFAULT NULL,
          is_default CHAR(1) NOT NULL DEFAULT 'N',
          status CHAR(1) NOT NULL DEFAULT '0',
          create_by VARCHAR(64) NOT NULL DEFAULT '',
          create_time DATETIME DEFAULT NULL,
          update_by VARCHAR(64) NOT NULL DEFAULT '',
          update_time DATETIME DEFAULT NULL,
          remark VARCHAR(500) DEFAULT NULL,
          PRIMARY KEY (dict_code)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_config (
          config_id INT NOT NULL AUTO_INCREMENT,
          config_name VARCHAR(100) NOT NULL DEFAULT '',
          config_key VARCHAR(100) NOT NULL DEFAULT '',
          config_value VARCHAR(500) NOT NULL DEFAULT '',
          config_type CHAR(1) NOT NULL DEFAULT 'N',
          create_by VARCHAR(64) NOT NULL DEFAULT '',
          create_time DATETIME DEFAULT NULL,
          update_by VARCHAR(64) NOT NULL DEFAULT '',
          update_time DATETIME DEFAULT NULL,
          remark VARCHAR(500) DEFAULT NULL,
          PRIMARY KEY (config_id),
          UNIQUE KEY uk_sys_config_key (config_key)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_notice (
          notice_id INT NOT NULL AUTO_INCREMENT,
          notice_title VARCHAR(50) NOT NULL,
          notice_type CHAR(1) NOT NULL,
          notice_content TEXT,
          status CHAR(1) NOT NULL DEFAULT '0',
          create_by VARCHAR(64) NOT NULL DEFAULT '',
          create_time DATETIME DEFAULT NULL,
          update_by VARCHAR(64) NOT NULL DEFAULT '',
          update_time DATETIME DEFAULT NULL,
          remark VARCHAR(255) DEFAULT NULL,
          PRIMARY KEY (notice_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_oper_log (
          oper_id BIGINT NOT NULL AUTO_INCREMENT,
          title VARCHAR(50) NOT NULL DEFAULT '',
          business_type INT NOT NULL DEFAULT 0,
          method VARCHAR(100) NOT NULL DEFAULT '',
          request_method VARCHAR(10) NOT NULL DEFAULT '',
          operator_type INT NOT NULL DEFAULT 1,
          oper_name VARCHAR(50) NOT NULL DEFAULT '',
          dept_name VARCHAR(50) NOT NULL DEFAULT '',
          oper_url VARCHAR(255) NOT NULL DEFAULT '',
          oper_ip VARCHAR(128) NOT NULL DEFAULT '',
          oper_location VARCHAR(255) NOT NULL DEFAULT '',
          oper_param VARCHAR(2000) NOT NULL DEFAULT '',
          json_result VARCHAR(2000) NOT NULL DEFAULT '',
          status INT NOT NULL DEFAULT 0,
          error_msg VARCHAR(2000) NOT NULL DEFAULT '',
          oper_time DATETIME DEFAULT NULL,
          cost_time BIGINT NOT NULL DEFAULT 0,
          PRIMARY KEY (oper_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_logininfor (
          info_id BIGINT NOT NULL AUTO_INCREMENT,
          user_name VARCHAR(50) NOT NULL DEFAULT '',
          ipaddr VARCHAR(128) NOT NULL DEFAULT '',
          login_location VARCHAR(255) NOT NULL DEFAULT '',
          browser VARCHAR(50) NOT NULL DEFAULT '',
          os VARCHAR(50) NOT NULL DEFAULT '',
          status CHAR(1) NOT NULL DEFAULT '0',
          msg VARCHAR(255) NOT NULL DEFAULT '',
          login_time DATETIME DEFAULT NULL,
          PRIMARY KEY (info_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
        """
        CREATE TABLE IF NOT EXISTS sys_session (
          token VARCHAR(80) NOT NULL,
          user_id BIGINT NOT NULL,
          user_name VARCHAR(30) NOT NULL,
          ipaddr VARCHAR(128) NOT NULL DEFAULT '',
          user_agent VARCHAR(200) NOT NULL DEFAULT '',
          csrf VARCHAR(80) NOT NULL,
          surface VARCHAR(16) NOT NULL,
          login_time DATETIME NOT NULL,
          last_access DATETIME NOT NULL,
          PRIMARY KEY (token),
          KEY idx_sys_session_user (user_id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
        """,
    )

    # menu_id, name, parent, order, path, type, perms
    MENUS = (
        (1, "System", 0, 1, "system", "M", ""),
        (100, "User", 1, 1, "user", "C", "system:user:list"),
        (1001, "User query", 100, 1, "", "F", "system:user:query"),
        (1002, "User add", 100, 2, "", "F", "system:user:add"),
        (1003, "User edit", 100, 3, "", "F", "system:user:edit"),
        (1004, "User remove", 100, 4, "", "F", "system:user:remove"),
        (1005, "Reset password", 100, 5, "", "F", "system:user:resetPwd"),
        (101, "Role", 1, 2, "role", "C", "system:role:list"),
        (1011, "Role query", 101, 1, "", "F", "system:role:query"),
        (1012, "Role edit", 101, 2, "", "F", "system:role:edit"),
        (102, "Menu", 1, 3, "menu", "C", "system:menu:list"),
        (1021, "Menu add", 102, 1, "", "F", "system:menu:add"),
        (103, "Department", 1, 4, "dept", "C", "system:dept:list"),
        (1031, "Department add", 103, 1, "", "F", "system:dept:add"),
        (104, "Post", 1, 5, "post", "C", "system:post:list"),
        (1041, "Post add", 104, 1, "", "F", "system:post:add"),
        (2, "Setup", 0, 2, "setup", "M", ""),
        (110, "Parameter", 2, 1, "config", "C", "system:config:list"),
        (1101, "Parameter edit", 110, 1, "", "F", "system:config:edit"),
        (1102, "Parameter add", 110, 2, "", "F", "system:config:add"),
        (111, "Dictionary", 2, 2, "dict", "C", "system:dict:list"),
        (1111, "Dictionary add", 111, 1, "", "F", "system:dict:add"),
        (112, "Notice", 2, 3, "notice", "C", "system:notice:list"),
        (1121, "Notice add", 112, 1, "", "F", "system:notice:add"),
        (3, "Audit", 0, 3, "audit", "M", ""),
        (120, "Operation log", 3, 1, "operlog", "C", "monitor:operlog:list"),
        (121, "Login log", 3, 2, "logininfor", "C", "monitor:logininfor:list"),
        (122, "Online", 3, 3, "online", "C", "monitor:online:list"),
        (1221, "Force logout", 122, 1, "", "F", "monitor:online:forceLogout"),
    )

    COMMON_MENUS = (1, 100, 1001, 2, 110, 112, 3, 120, 121)
    LEAD_MENUS = (1, 100, 1001)

    def __init__(self, store, passwords=None, logger=None):
        self.logger = logger
        if logger is not None:
            logger.log_message("instantiated", component="Schema")
        self.store = store
        self.passwords = passwords or PasswordHash(logger=logger)

    def statements(self) -> tuple:
        """General Purpose: MySQL DDL, or the sqlite3 equivalent of those columns."""
        engine = "mysql"
        settings = getattr(self.store, "settings", None)
        if settings is not None:
            engine = settings.engine()
        if engine == "sqlite3":
            return tuple(self.sqlite_statement(item) for item in self.STATEMENTS)
        return self.STATEMENTS

    @staticmethod
    def sqlite_statement(sql: str) -> str:
        """General Purpose: One CREATE TABLE that sqlite3 accepts. No foreign key."""
        text = " ".join(sql.split())
        text = text.replace(" ENGINE=InnoDB DEFAULT CHARSET=utf8mb4", "")
        text = re.sub(
            r"\b(BIGINT|INT)\s+NOT NULL\s+AUTO_INCREMENT",
            "INTEGER PRIMARY KEY AUTOINCREMENT",
            text,
        )
        text = re.sub(r"UNIQUE KEY \w+ \(([^)]+)\)", r"UNIQUE (\1)", text)
        text = re.sub(r",\s*KEY \w+ \([^)]+\)", "", text)
        for name in re.findall(r"(\w+)\s+INTEGER PRIMARY KEY AUTOINCREMENT", text):
            text = re.sub(
                r",\s*PRIMARY KEY \({0}\)".format(re.escape(name)),
                "",
                text,
            )
        text = re.sub(r",\s*\)", ")", text)
        return text

    def ensure(self) -> str:
        """General Purpose: Create tables. Seed only when no user row exists."""
        for statement in self.statements():
            self.store.execute(statement)
        row = self.store.query_one("SELECT COUNT(*) AS n FROM sys_user")
        count = 0 if row is None else int(row["n"])
        if count:
            return "present"
        self._seed()
        return "seeded"

    def _seed(self) -> None:
        passwords = self.passwords
        admin_hash = passwords.hash(self.INITIAL_ADMIN_PASSWORD)
        operator_hash = passwords.hash(self.INITIAL_OPERATOR_PASSWORD)
        lead_hash = passwords.hash(self.INITIAL_LEAD_PASSWORD)
        clerk_hash = passwords.hash(self.INITIAL_CLERK_PASSWORD)

        def work(run):
            now = run("SELECT NOW() AS t")["rows"][0]["t"]
            depts = (
                (100, 0, "0", "Group", 0),
                (101, 100, "0,100", "Engineering", 1),
                (102, 100, "0,100", "Operations", 2),
                (103, 101, "0,100,101", "Platform", 3),
            )
            for item in depts:
                run(
                    """
                    INSERT INTO sys_dept
                      (dept_id, parent_id, ancestors, dept_name, order_num,
                       status, del_flag, create_by, create_time)
                    VALUES (%s, %s, %s, %s, %s, '0', '0', 'schema', %s)
                    """,
                    (item[0], item[1], item[2], item[3], item[4], now),
                )
            users = (
                (1, 101, "admin", "Administrator", admin_hash),
                (2, 102, "operator", "Operator", operator_hash),
                (3, 103, "clerk", "Clerk", clerk_hash),
                (4, 101, "lead", "Lead", lead_hash),
            )
            for item in users:
                run(
                    """
                    INSERT INTO sys_user
                      (user_id, dept_id, user_name, nick_name, user_type, password,
                       status, del_flag, create_by, create_time, pwd_update_date)
                    VALUES (%s, %s, %s, %s, '00', %s, '0', '0', 'schema', %s, %s)
                    """,
                    (item[0], item[1], item[2], item[3], item[4], now, now),
                )
            posts = (
                (1, "ceo", "Chief", 1),
                (2, "se", "Engineer", 2),
                (3, "hr", "People", 3),
                (4, "user", "Staff", 4),
            )
            for item in posts:
                run(
                    """
                    INSERT INTO sys_post
                      (post_id, post_code, post_name, post_sort, status, create_by, create_time)
                    VALUES (%s, %s, %s, %s, '0', 'schema', %s)
                    """,
                    (item[0], item[1], item[2], item[3], now),
                )
            roles = (
                (1, "Super administrator", "admin", 1, "1", "super administrator"),
                (2, "Common", "common", 2, "3", "this department"),
                (3, "Lead", "lead", 3, "4", "this department and children"),
            )
            for item in roles:
                run(
                    """
                    INSERT INTO sys_role
                      (role_id, role_name, role_key, role_sort, data_scope,
                       menu_check_strictly, dept_check_strictly, status, del_flag,
                       create_by, create_time, remark)
                    VALUES (%s, %s, %s, %s, %s, 1, 1, '0', '0', 'schema', %s, %s)
                    """,
                    (item[0], item[1], item[2], item[3], item[4], now, item[5]),
                )
            for menu in self.MENUS:
                run(
                    """
                    INSERT INTO sys_menu
                      (menu_id, menu_name, parent_id, order_num, path, menu_type,
                       visible, status, perms, create_by, create_time)
                    VALUES (%s, %s, %s, %s, %s, %s, '0', '0', %s, 'schema', %s)
                    """,
                    (menu[0], menu[1], menu[2], menu[3], menu[4], menu[5], menu[6], now),
                )
            links = ((1, 1), (2, 2), (3, 2), (4, 3))
            for user_id, role_id in links:
                run(
                    "INSERT INTO sys_user_role (user_id, role_id) VALUES (%s, %s)",
                    (user_id, role_id),
                )
            for menu in self.MENUS:
                run(
                    "INSERT INTO sys_role_menu (role_id, menu_id) VALUES (1, %s)",
                    (menu[0],),
                )
            for menu_id in self.COMMON_MENUS:
                run(
                    "INSERT INTO sys_role_menu (role_id, menu_id) VALUES (2, %s)",
                    (menu_id,),
                )
            for menu_id in self.LEAD_MENUS:
                run(
                    "INSERT INTO sys_role_menu (role_id, menu_id) VALUES (3, %s)",
                    (menu_id,),
                )
            configs = (
                (1, "Skin name", "sys.index.skinName", "skin-blue", "Y"),
                (2, "Initial password", "sys.user.initPassword", "123456", "Y"),
                (3, "Captcha", "sys.account.captchaEnabled", "false", "Y"),
                (4, "Registration", "sys.account.registerUser", "false", "Y"),
            )
            for item in configs:
                run(
                    """
                    INSERT INTO sys_config
                      (config_id, config_name, config_key, config_value, config_type,
                       create_by, create_time)
                    VALUES (%s, %s, %s, %s, %s, 'schema', %s)
                    """,
                    (item[0], item[1], item[2], item[3], item[4], now),
                )
            types = (
                (1, "User sex", "sys_user_sex"),
                (2, "Normal or disabled", "sys_normal_disable"),
                (3, "Notice type", "sys_notice_type"),
            )
            for item in types:
                run(
                    """
                    INSERT INTO sys_dict_type
                      (dict_id, dict_name, dict_type, status, create_by, create_time)
                    VALUES (%s, %s, %s, '0', 'schema', %s)
                    """,
                    (item[0], item[1], item[2], now),
                )
            data_rows = (
                (1, 1, "Male", "0", "sys_user_sex", "Y"),
                (2, 2, "Female", "1", "sys_user_sex", "N"),
                (3, 3, "Unknown", "2", "sys_user_sex", "N"),
                (4, 1, "Normal", "0", "sys_normal_disable", "Y"),
                (5, 2, "Disabled", "1", "sys_normal_disable", "N"),
                (6, 1, "Notice", "1", "sys_notice_type", "Y"),
                (7, 2, "Announcement", "2", "sys_notice_type", "N"),
            )
            for item in data_rows:
                run(
                    """
                    INSERT INTO sys_dict_data
                      (dict_code, dict_sort, dict_label, dict_value, dict_type,
                       is_default, status, create_by, create_time)
                    VALUES (%s, %s, %s, %s, %s, %s, '0', 'schema', %s)
                    """,
                    (item[0], item[1], item[2], item[3], item[4], item[5], now),
                )
            run(
                """
                INSERT INTO sys_notice
                  (notice_id, notice_title, notice_type, notice_content, status,
                   create_by, create_time)
                VALUES (1, 'Welcome', '2', 'PyRbac is ready.', '0', 'schema', %s)
                """,
                (now,),
            )
            return None

        self.store.transaction(work)
        if self.logger is not None:
            self.logger.log_message("seeded rbac tables", component="Schema")
