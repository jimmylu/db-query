# Database Query Tool

一个现代化的数据库查询工具，支持 PostgreSQL 和 MySQL 数据库连接、元数据查看、SQL 查询执行和自然语言查询。

## 功能特性

- 🔌 **数据库连接管理**: 支持 PostgreSQL 和 MySQL 数据库连接，连接信息本地存储
- 📊 **元数据查看**: 自动检索和显示数据库表、视图和列信息
- 🔍 **SQL 查询执行**: 安全的 SQL SELECT 查询执行，自动添加 LIMIT 限制
- 🤖 **自然语言查询**: 使用 LLM 将自然语言问题转换为 SQL 查询（支持数据库特定语法）
- 🔒 **安全验证**: 仅允许 SELECT 查询，防止数据修改和 SQL 注入
- ⚡ **性能优化**: 查询超时控制、连接超时处理、元数据缓存、连接池

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

详细 API 文档请参考 `specs/001-db-query-tool/contracts/openapi.yaml`

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

