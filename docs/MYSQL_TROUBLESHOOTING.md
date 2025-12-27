# MySQL 数据库故障排除指南

本指南帮助您诊断和解决使用 MySQL 数据库时遇到的常见问题。

## 目录

1. [连接问题](#连接问题)
2. [元数据提取问题](#元数据提取问题)
3. [查询执行问题](#查询执行问题)
4. [性能问题](#性能问题)
5. [数据类型问题](#数据类型问题)

---

## 连接问题

### 问题：无法连接到 MySQL 数据库

**症状**:
- 错误信息：`Connection refused` 或 `Access denied`
- 连接状态显示为 `error` 或 `disconnected`

**可能原因和解决方案**:

#### 1. 检查连接 URL 格式

**正确格式**:
```
mysql://username:password@host:port/database
```

**示例**:
```
mysql://root:password123@localhost:3306/todolist
```

**常见错误**:
- ❌ `mysql:/root:password@localhost/db` (缺少斜杠)
- ❌ `mysql://user@localhost/db` (缺少密码)
- ❌ `mysql://user:pass@localhost` (缺少数据库名)
- ✅ `mysql://user:pass@localhost:3306/db`

#### 2. 验证 MySQL 服务器运行状态

```bash
# 检查 MySQL 是否运行
sudo systemctl status mysql  # Linux
# 或
brew services list | grep mysql  # macOS

# 检查端口是否开放
lsof -i :3306
# 或
netstat -an | grep 3306
```

#### 3. 验证用户权限

```sql
-- 连接到 MySQL
mysql -u root -p

-- 检查用户权限
SHOW GRANTS FOR 'your_username'@'localhost';

-- 如果需要，授予权限
GRANT SELECT ON database_name.* TO 'your_username'@'localhost';
FLUSH PRIVILEGES;
```

#### 4. 检查防火墙设置

```bash
# Linux (ufw)
sudo ufw allow 3306/tcp

# Linux (firewalld)
sudo firewall-cmd --add-port=3306/tcp --permanent
sudo firewall-cmd --reload
```

#### 5. Docker 环境问题

如果使用 Docker 运行 MySQL：

```bash
# 检查容器状态
docker ps -a | grep mysql

# 检查容器日志
docker logs mysql-container-name

# 确保端口映射正确
docker inspect mysql-container-name | grep -A 10 "PortBindings"
```

---

## 元数据提取问题

### 问题：元数据为空或不完整

**症状**:
- 表列表为空
- 缺少某些表或视图
- 列信息不完整

**解决方案**:

#### 1. 验证数据库权限

```sql
-- 需要 SELECT 权限访问 information_schema
GRANT SELECT ON information_schema.* TO 'your_username'@'localhost';
```

#### 2. 刷新元数据缓存

在 API 请求中添加 `?refresh=true` 参数：

```bash
curl -X GET "http://localhost:3000/api/connections/{id}/metadata?refresh=true"
```

#### 3. 检查系统表过滤

应用会自动过滤以下系统数据库：
- `information_schema`
- `mysql`
- `performance_schema`
- `sys`

如果您的表在这些数据库中，它们不会显示。

#### 4. 验证表存在

```sql
-- 查看所有数据库
SHOW DATABASES;

-- 查看特定数据库的表
USE your_database;
SHOW TABLES;

-- 查看表结构
DESCRIBE table_name;
```

---

## 查询执行问题

### 问题：查询被拒绝

**症状**:
- 错误：`Only SELECT queries are permitted`
- 非 SELECT 语句被阻止

**解决方案**:

这是**预期行为**。出于安全考虑，系统只允许 SELECT 查询。

**允许的查询类型**:
- ✅ `SELECT * FROM users`
- ✅ `SELECT id, name FROM products WHERE price > 100`
- ✅ `SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id`

**不允许的查询类型**:
- ❌ `INSERT INTO users VALUES (...)`
- ❌ `UPDATE users SET name = '...'`
- ❌ `DELETE FROM users WHERE id = 1`
- ❌ `DROP TABLE users`
- ❌ `CREATE TABLE ...`

**注意**: 这是 Constitution 安全原则的要求，无法禁用。

### 问题：查询超时

**症状**:
- 错误：`Query execution timeout`
- 查询运行时间超过 30 秒

**解决方案**:

#### 1. 优化查询

```sql
-- 使用索引
CREATE INDEX idx_user_id ON orders(user_id);

-- 添加 WHERE 子句减少数据量
SELECT * FROM large_table WHERE created_at > '2024-01-01' LIMIT 1000;

-- 避免全表扫描
EXPLAIN SELECT * FROM users WHERE email = 'test@example.com';
```

#### 2. 增加 LIMIT

系统会自动添加 `LIMIT 1000`，但您可以手动指定更小的值：

```sql
SELECT * FROM large_table LIMIT 100;
```

### 问题：LIMIT 未应用

**症状**:
- `limit_applied` 字段为 `false`
- 查询返回超过预期的行数

**解释**:

- 如果查询已包含 LIMIT 子句，系统不会再添加
- 如果查询没有 LIMIT，系统会自动添加 `LIMIT 1000`

**示例**:

```sql
-- 这个查询不会自动添加 LIMIT
SELECT * FROM users LIMIT 50;  -- limit_applied = false

-- 这个查询会自动添加 LIMIT 1000
SELECT * FROM users;  -- limit_applied = true
```

---

## 性能问题

### 问题：查询执行缓慢

**诊断步骤**:

#### 1. 使用 EXPLAIN 分析查询

```sql
EXPLAIN SELECT * FROM users WHERE email = 'test@example.com';
```

检查：
- **type**: `ALL` 表示全表扫描（慢），`ref` 或 `const` 表示使用索引（快）
- **rows**: 扫描的行数
- **Extra**: 是否使用 `Using index` 或 `Using where`

#### 2. 添加索引

```sql
-- 为常用查询字段添加索引
CREATE INDEX idx_email ON users(email);
CREATE INDEX idx_status ON todos(status);
CREATE INDEX idx_user_category ON todos(user_id, category_id);
```

#### 3. 检查连接池状态

确保连接池配置合理：

```rust
// backend/src/services/database/mysql.rs
// 默认配置：
// - min_connections: 2
// - max_connections: 10
```

#### 4. 监控 MySQL 性能

```sql
-- 查看慢查询
SHOW VARIABLES LIKE 'slow_query%';

-- 查看当前连接
SHOW PROCESSLIST;

-- 查看表状态
SHOW TABLE STATUS FROM database_name;
```

---

## 数据类型问题

### 问题：数据类型转换错误

**症状**:
- JSON 结果中的值格式不正确
- 时间戳显示异常
- 数字显示为字符串

**支持的 MySQL 数据类型**:

#### 数值类型
- `TINYINT`, `SMALLINT`, `MEDIUMINT`, `INT`, `BIGINT`
- `DECIMAL`, `NUMERIC`, `FLOAT`, `DOUBLE`
- 转换为 JSON 数字类型

#### 字符串类型
- `VARCHAR`, `CHAR`, `TEXT`, `MEDIUMTEXT`, `LONGTEXT`
- 转换为 JSON 字符串类型

#### 日期和时间类型
- `DATE`, `DATETIME`, `TIMESTAMP`, `TIME`, `YEAR`
- 转换为 JSON 字符串（ISO 8601 格式）

#### 二进制类型
- `BLOB`, `BINARY`, `VARBINARY`
- 转换为 Base64 编码字符串

#### 特殊类型
- `ENUM`: 转换为字符串值
- `SET`: 转换为逗号分隔的字符串
- `JSON`: 直接保持 JSON 格式

**示例**:

```sql
-- MySQL 查询
SELECT
    id,              -- INT -> "1"
    price,           -- DECIMAL -> "19.99"
    created_at,      -- TIMESTAMP -> "2025-12-27 10:30:00"
    status,          -- ENUM('pending','active') -> "pending"
    is_active        -- TINYINT(1) -> "1" 或 "0"
FROM products;
```

### 问题：ENUM 和 SET 类型处理

**ENUM 类型**:
```sql
-- 定义
CREATE TABLE tasks (
    status ENUM('pending', 'in_progress', 'completed')
);

-- 查询结果
{
    "status": "pending"  -- 字符串值
}
```

**SET 类型**:
```sql
-- 定义
CREATE TABLE permissions (
    access SET('read', 'write', 'delete')
);

-- 查询结果
{
    "access": "read,write"  -- 逗号分隔的字符串
}
```

---

## 自然语言查询问题

### 问题：LLM 生成的 SQL 语法错误

**症状**:
- 自然语言查询返回错误
- 生成的 SQL 无法执行

**解决方案**:

#### 1. 检查 LLM 服务配置

```bash
# 验证 LLM 服务可访问
curl -X GET http://localhost:8080/health

# 检查环境变量
echo $LLM_GATEWAY_URL
echo $LLM_API_KEY
```

#### 2. 提供更清晰的问题描述

**好的示例**:
- ✅ "显示所有活跃用户的邮箱地址"
- ✅ "查找本周创建的待办事项数量"
- ✅ "按分类统计待办事项"

**不好的示例**:
- ❌ "用户" (太模糊)
- ❌ "数据" (不明确要查询什么)
- ❌ "复杂的联表查询统计分析" (太复杂)

#### 3. MySQL 特定语法提示

LLM 已配置支持 MySQL 语法，包括：

**日期函数**:
```sql
NOW()
CURDATE()
DATE_SUB(CURDATE(), INTERVAL 7 DAY)
```

**字符串函数**:
```sql
CONCAT(first_name, ' ', last_name)
```

**标识符引用**:
```sql
SELECT `user-name` FROM `my-table`  -- 使用反引号
```

---

## 常见错误代码

| 错误代码 | 含义 | 解决方案 |
|---------|------|---------|
| 1045 | Access denied | 检查用户名和密码 |
| 1049 | Unknown database | 验证数据库名称 |
| 2003 | Can't connect | 检查 MySQL 服务和网络 |
| 1054 | Unknown column | 验证列名拼写 |
| 1064 | SQL syntax error | 检查 SQL 语法 |
| 1146 | Table doesn't exist | 验证表名 |
| 1205 | Lock wait timeout | 减少并发或优化查询 |

---

## 调试技巧

### 1. 启用详细日志

```bash
# 后端日志
cd backend
RUST_LOG=debug cargo run

# 查看查询日志
tail -f server.log | grep "Executing query"
```

### 2. 使用 REST 客户端测试

创建 `test.rest` 文件：

```http
### 创建 MySQL 连接
POST http://localhost:3000/api/connections
Content-Type: application/json

{
  "connection_url": "mysql://root:password123@localhost:3306/todolist",
  "name": "Test MySQL",
  "database_type": "mysql"
}

### 测试查询
POST http://localhost:3000/api/connections/{{connection_id}}/query
Content-Type: application/json

{
  "query": "SELECT * FROM users LIMIT 5"
}
```

### 3. 验证数据库连接

```bash
# 使用 MySQL 客户端测试
mysql -h localhost -u root -p -D todolist

# 执行测试查询
SELECT VERSION();
SELECT DATABASE();
SHOW TABLES;
```

---

## 获取帮助

如果问题仍未解决：

1. **检查日志**: 查看后端日志文件获取详细错误信息
2. **查看文档**: 参考 `README.md` 和 `CLAUDE.md` 获取更多信息
3. **提交 Issue**: 在 GitHub 仓库创建 issue，包含：
   - 错误消息
   - MySQL 版本
   - 连接 URL（隐藏敏感信息）
   - 查询语句
   - 完整的错误堆栈

---

## 最佳实践

### 连接管理
- ✅ 使用连接池而不是单个连接
- ✅ 定期测试连接健康状态
- ✅ 为不同环境使用不同的连接

### 查询优化
- ✅ 始终使用 LIMIT 子句
- ✅ 为常用查询字段添加索引
- ✅ 避免 `SELECT *`，明确指定需要的列
- ✅ 使用 EXPLAIN 分析慢查询

### 安全性
- ✅ 使用最小权限原则
- ✅ 定期更换数据库密码
- ✅ 不在 URL 中暴露敏感信息
- ✅ 使用环境变量存储凭证

### 性能监控
- ✅ 监控查询执行时间
- ✅ 跟踪慢查询日志
- ✅ 定期分析表统计信息
- ✅ 使用 MySQL Performance Schema

---

## 相关资源

- [MySQL 官方文档](https://dev.mysql.com/doc/)
- [项目 README](../README.md)
- [MySQL 连接示例](../fixtures/MYSQL_TODOLIST.md)
- [API 文档](../specs/001-db-query-tool/contracts/openapi.yaml)
- [Constitution](../.specify/memory/constitution.md)
