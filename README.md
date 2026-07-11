# Database Query Tool

一个现代化的数据库查询工具，支持 PostgreSQL 和 MySQL 数据库连接、元数据查看、SQL 查询执行、自然语言查询，以及**跨数据库查询**功能。

## 功能特性

### 核心功能

- 🔌 **数据库连接管理**: 支持 PostgreSQL 和 MySQL 数据库连接，连接信息本地存储
- 📊 **元数据查看**: 自动检索和显示数据库表、视图和列信息
- 🔍 **SQL 查询执行**: 安全的 SQL SELECT 查询执行，自动添加 LIMIT 限制
- 🤖 **自然语言查询**: 使用 LLM 将自然语言问题转换为 SQL 查询（支持数据库特定语法）
- 🔒 **安全验证**: 仅允许 SELECT 查询，防止数据修改和 SQL 注入
- ⚡ **性能优化**: 查询超时控制、连接超时处理、元数据缓存、连接池

### 🆕 跨数据库查询 (Phase 4)

- 🔗 **跨数据库 JOIN**: 在多个数据库之间执行 JOIN 查询（MySQL ↔ PostgreSQL ↔ 其他）
- 🏷️ **数据库别名系统**: 使用简单的别名（db1, db2）代替长 UUID 连接标识符
- ⚡ **智能查询优化**: 自动检测单数据库查询并优化执行（性能提升 89%）
- 📊 **执行详情展示**: 查看子查询执行详情、性能指标、数据源信息
- 🎯 **直观的 UI 界面**: 多数据库选择器、别名配置、示例查询模板
- ⏳ **UNION 查询支持**: 框架已就绪（60% 完成）

### 🎨 编辑器增强功能

- 🎯 **SQL 格式化**: 一键格式化 SQL 查询，自动缩进和关键字大写
- 🌓 **深色/亮色模式**: Monaco 编辑器主题切换，偏好设置本地保存
- 📝 **查询模板系统**: 8 个内置模板 + 自定义模板，支持分类管理
- ⌨️ **键盘快捷键**: Cmd/Ctrl + Enter 快速执行查询
- 📜 **查询历史**: 自动保存最近 50 条查询记录，支持快速加载
- 📊 **数据导出**: 支持导出为 CSV 和 JSON 格式
- 🔄 **虚拟滚动**: 大数据集性能优化，流畅显示千行数据

## 技术栈

### 后端
- **Rust** - 高性能系统编程语言
- **Axum** - 现代化的 Web 框架
- **Tokio** - 异步运行时
- **DataFusion** - SQL 查询引擎
- **SQLParser** - SQL 解析和验证
- **tokio-postgres** - PostgreSQL 客户端（带连接池）
- **mysql_async** - MySQL 客户端（带连接池）
- **rusqlite** - SQLite 元数据存储

### 前端
- **React 18** - UI 框架
- **Refine 5** - 企业级 React 框架
- **Ant Design** - UI 组件库
- **Monaco Editor** - SQL 编辑器
- **Vite** - 构建工具
- **TypeScript** - 类型安全

## 快速开始

### 前置要求

- Rust (latest stable)
- Node.js 18+ 和 npm/yarn
- PostgreSQL 或 MySQL 数据库（用于查询）

### 安装

1. **克隆仓库**:
   ```bash
   git clone <repository-url>
   cd db_query
   ```

2. **安装后端依赖**:
   ```bash
   cd backend
   cargo build
   ```

3. **安装前端依赖**:
   ```bash
   cd ../frontend
   npm install
   ```

### 配置

1. **后端配置** (`backend/.env`):
   ```env
   DATABASE_URL=./metadata.db
   SERVER_HOST=0.0.0.0
   SERVER_PORT=3000
   LLM_GATEWAY_URL=http://localhost:8080
   LLM_API_KEY=your-api-key-optional
   ```

2. **前端配置** (`frontend/.env`):
   ```env
   VITE_API_URL=http://localhost:3000/api
   ```

### 运行

使用 Makefile 快速启动：

```bash
# 安装所有依赖
make install

# 启动后端（端口 3000）
make dev-backend

# 启动前端（端口 5173）
make dev-frontend
```

或者手动启动：

```bash
# 后端
cd backend
cargo run

# 前端（新终端）
cd frontend
npm run dev
```

### 使用

1. 打开浏览器访问 `http://localhost:5173`
2. 在"数据库连接"表单中输入数据库连接 URL：
   - PostgreSQL: `postgresql://user:password@host:5432/database`
   - MySQL: `mysql://user:password@host:3306/database`
3. 连接成功后，查看数据库元数据（表、视图、列）
4. 在"查询"页面执行 SQL 查询或使用自然语言查询
5. **新功能**: 在"跨数据库查询"页面执行跨数据库 JOIN 查询

#### 跨数据库查询示例

访问 `http://localhost:5173/cross-database` 执行跨数据库查询：

```sql
-- 跨数据库 JOIN 示例
SELECT u.username, t.title
FROM db1.users u
JOIN db2.todos t ON u.id = t.user_id
WHERE t.status = 'pending'
LIMIT 10
```

**功能亮点**:
- 🔄 支持 MySQL、PostgreSQL 等多种数据库组合
- ⚡ 智能优化：单数据库查询自动优化（3ms 执行时间）
- 📊 详细的子查询执行信息和性能指标
- 💡 内置示例查询，快速上手

#### 使用 Docker 快速测试

**启动 MySQL 测试实例**:
```bash
docker run -d --name test-mysql \
  -e MYSQL_ROOT_PASSWORD=password \
  -e MYSQL_DATABASE=testdb \
  -p 3306:3306 \
  mysql:8.0

# 连接 URL: mysql://root:password@localhost:3306/testdb
```

**启动 PostgreSQL 测试实例**:
```bash
docker run -d --name test-postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=testdb \
  -p 5432:5432 \
  postgres:15

# 连接 URL: postgresql://postgres:password@localhost:5432/testdb
```

## API 文档

### 连接管理

- `GET /api/connections` - 列出所有连接
- `POST /api/connections` - 创建新连接
- `GET /api/connections/{id}` - 获取连接详情
- `DELETE /api/connections/{id}` - 删除连接

### 元数据

- `GET /api/connections/{id}/metadata?refresh=true` - 获取元数据（可选强制刷新）

### 查询

- `POST /api/connections/{id}/query` - 执行 SQL 查询
- `POST /api/connections/{id}/nl-query` - 执行自然语言查询

### 🆕 跨数据库查询

- `POST /api/cross-database/query` - 执行跨数据库 JOIN/UNION 查询

**请求示例**:
```json
{
  "query": "SELECT u.username, t.title FROM db1.users u JOIN db2.todos t ON u.id = t.user_id",
  "connection_ids": ["uuid-1", "uuid-2"],
  "database_aliases": {
    "db1": "uuid-1",
    "db2": "uuid-2"
  },
  "timeout_secs": 60,
  "apply_limit": true,
  "limit_value": 1000
}
```

**响应示例**:
```json
{
  "original_query": "SELECT ...",
  "sub_queries": [
    {
      "connection_id": "uuid-1",
      "database_type": "mysql",
      "query": "SELECT * FROM users",
      "row_count": 100,
      "execution_time_ms": 10
    }
  ],
  "results": [...],
  "row_count": 50,
  "execution_time_ms": 25,
  "limit_applied": false,
  "executed_at": "2025-12-27T..."
}
```

详细 API 文档请参考：
- `specs/001-db-query-tool/contracts/openapi.yaml`
- `backend/CROSS_DATABASE_QUICKSTART.md` - 跨数据库查询快速指南
- `frontend/CROSS_DATABASE_UI_GUIDE.md` - UI 使用指南

## 开发

### 项目结构

```
db_query/
├── backend/          # Rust 后端
│   ├── src/
│   │   ├── api/      # API 处理器和路由
│   │   ├── models/   # 数据模型
│   │   ├── services/ # 业务逻辑
│   │   ├── storage/   # 存储层
│   │   └── validation/# SQL 验证
│   └── Cargo.toml
├── frontend/         # React 前端
│   ├── src/
│   │   ├── components/ # React 组件
│   │   ├── pages/      # 页面组件
│   │   ├── services/   # API 客户端
│   │   └── types/      # TypeScript 类型
│   └── package.json
└── specs/            # 规范和文档
```

### 代码质量

```bash
# 后端
make lint-backend
make format-backend
make test-backend

# 前端
make lint-frontend
make format-frontend
make test-frontend
```

### 测试 API

使用 VS Code REST Client 测试 API（见 `fixtures/test.rest`）

## 安全特性

- ✅ 仅允许 SELECT 查询
- ✅ SQL 注入防护（SQLParser 验证）
- ✅ 自动 LIMIT 限制（默认 1000 行）
- ✅ 连接超时控制（10 秒）
- ✅ 查询执行超时控制（30 秒）
- ✅ 输入验证和清理

## 许可证

[添加许可证信息]

## 贡献

欢迎提交 Issue 和 Pull Request！

## Contributors

We sincerely thank the following contributors for their time, ideas, and dedication to this project:

- jimmylu <mr.jimmylu@gmail.comn>
- pipilu <mr.jimmylu@gmail.com>

Your contributions continue to make this project stronger and more valuable to the community.
