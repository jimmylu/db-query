# Rust 编码设计原则

## 目录

1. [核心 Rust 哲学](#1-核心-rust-哲学)
2. [异步并发编程模式](#2-异步并发编程模式)
3. [错误处理](#3-错误处理)
4. [类型系统与 API 设计](#4-类型系统与-api-设计)
5. [性能与优化](#5-性能与优化)
6. [Web 服务开发 (Axum/Tokio)](#6-web-服务开发-axumtokio)
7. [数据库集成](#7-数据库集成)
8. [测试与文档](#8-测试与文档)
9. [项目结构](#9-项目结构)
10. [代码质量](#10-代码质量)

---

## 1. 核心 Rust 哲学

### 1.1 所有权 (Ownership)

**原则**: 每个值都有一个所有者，当所有者离开作用域时，值将被销毁。

```rust
// ✅ 好的实践：转移所有权给需要它的地方
fn process_connection(connection: DatabaseConnection) -> Result<(), AppError> {
    // connection 被消费，调用者无法再使用它
    connection.execute_query("SELECT 1")
}

// ✅ 好的实践：借用而不是转移所有权
fn validate_connection(connection: &DatabaseConnection) -> bool {
    // 只读访问，不获取所有权
    connection.is_valid()
}

// ❌ 避免：不必要的克隆
fn bad_process(connection: DatabaseConnection) -> Result<(), AppError> {
    let conn_clone = connection.clone(); // 昂贵的操作
    conn_clone.execute_query("SELECT 1")
}
```

**项目实例**:
```rust
// backend/src/services/db_service.rs
pub struct DatabaseService {
    connections: Arc<RwLock<HashMap<Uuid, DatabaseConnection>>>,
}

impl DatabaseService {
    // 借用 self，不转移所有权
    pub async fn get_connection(&self, id: &Uuid) -> Option<DatabaseConnection> {
        let connections = self.connections.read().await;
        connections.get(id).cloned() // 仅在需要时克隆
    }
}
```

### 1.2 借用 (Borrowing)

**原则**: 在任何给定时间，要么有一个可变引用，要么有任意数量的不可变引用。

```rust
// ✅ 好的实践：不可变借用用于读取
async fn read_metadata(cache: &MetadataCache, conn_id: &Uuid) -> Option<Metadata> {
    cache.get(conn_id).await
}

// ✅ 好的实践：可变借用用于修改
async fn update_metadata(cache: &mut MetadataCache, conn_id: Uuid, metadata: Metadata) {
    cache.insert(conn_id, metadata).await;
}

// ❌ 避免：同时持有可变和不可变引用
fn bad_borrow(data: &mut Vec<String>) {
    let first = &data[0];        // 不可变借用
    data.push("new".to_string()); // 可变借用 - 编译错误！
    println!("{}", first);
}
```

**并发场景中的借用**:
```rust
// ✅ 使用 Arc + RwLock 实现多读单写
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ConnectionPool {
    connections: Arc<RwLock<HashMap<Uuid, Connection>>>,
}

impl ConnectionPool {
    pub async fn get(&self, id: &Uuid) -> Option<Connection> {
        // 多个任务可以同时读取
        let lock = self.connections.read().await;
        lock.get(id).cloned()
    }

    pub async fn insert(&self, id: Uuid, conn: Connection) {
        // 独占写入
        let mut lock = self.connections.write().await;
        lock.insert(id, conn);
    }
}
```

### 1.3 生命周期 (Lifetimes)

**原则**: 明确引用的有效期，防止悬垂指针。

```rust
// ✅ 好的实践：显式生命周期注解
pub struct QueryContext<'a> {
    connection: &'a DatabaseConnection,
    metadata: &'a Metadata,
}

impl<'a> QueryContext<'a> {
    pub fn new(connection: &'a DatabaseConnection, metadata: &'a Metadata) -> Self {
        Self { connection, metadata }
    }

    pub fn execute(&self, sql: &str) -> Result<QueryResult, AppError> {
        // 'a 确保 connection 和 metadata 在整个执行过程中有效
        self.connection.execute(sql)
    }
}

// ✅ 生命周期省略规则
// 编译器可以推断出返回值的生命周期与输入相同
fn get_first_column(metadata: &Metadata) -> Option<&str> {
    metadata.tables.first().and_then(|t| t.columns.first().map(|c| c.name.as_str()))
}
```

### 1.4 零成本抽象 (Zero-Cost Abstractions)

**原则**: 高级抽象不应该带来运行时开销。

```rust
// ✅ 好的实践：使用迭代器链，编译后等同于手写循环
pub fn filter_large_tables(metadata: &Metadata, min_rows: u64) -> Vec<&str> {
    metadata.tables
        .iter()
        .filter(|table| table.estimated_rows > min_rows)
        .map(|table| table.name.as_str())
        .collect()
}

// ✅ 好的实践：泛型在编译时单态化，无虚函数调用开销
pub fn execute_with_adapter<A: DatabaseAdapter>(
    adapter: &A,
    query: &str,
) -> Result<QueryResult, AppError> {
    adapter.execute_query(query)
}

// ✅ 好的实践：使用 impl Trait 避免装箱
pub fn stream_results(query: String) -> impl Stream<Item = Result<Row, Error>> {
    // 返回具体类型，无动态分发
    tokio_stream::iter(vec![Ok(Row::default())])
}
```

### 1.5 内存安全无 GC

**原则**: 编译时保证内存安全，无运行时垃圾回收。

```rust
// ✅ RAII 模式：资源在离开作用域时自动清理
pub struct DatabaseConnection {
    client: sqlx::PgPool,
}

impl Drop for DatabaseConnection {
    fn drop(&mut self) {
        // 自动关闭连接池
        tracing::info!("Closing database connection");
    }
}

// ✅ 使用 Option 表示可能不存在的值
pub struct CachedMetadata {
    data: Option<Metadata>,
    last_updated: Option<SystemTime>,
}

impl CachedMetadata {
    pub fn is_stale(&self, ttl: Duration) -> bool {
        match self.last_updated {
            Some(time) => time.elapsed().unwrap_or_default() > ttl,
            None => true,
        }
    }
}
```

---

## 2. 异步并发编程模式

### 2.1 Tokio 运行时最佳实践

**原则**: 理解多线程运行时的工作窃取调度器，避免阻塞。

```rust
// ✅ 好的实践：main 函数正确配置 Tokio
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用默认多线程运行时
    let app = create_app().await?;
    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

// ✅ 自定义运行时配置
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 4 个工作线程
    run_server().await
}

// ✅ 使用 spawn_blocking 处理 CPU 密集或阻塞操作
async fn process_large_query_result(data: Vec<Row>) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        // CPU 密集型序列化操作
        serde_json::to_string(&data)
    })
    .await
    .map_err(|e| AppError::InternalError(e.to_string()))?
    .map_err(|e| AppError::SerializationError(e.to_string()))
}

// ❌ 避免：在异步上下文中阻塞
async fn bad_blocking_call() {
    std::thread::sleep(Duration::from_secs(1)); // 阻塞整个工作线程！
}
```

### 2.2 Async/Await 模式

**原则**: 使用 .await 暂停异步操作，释放线程给其他任务。

```rust
// ✅ 好的实践：并发执行独立任务
pub async fn fetch_all_metadata(
    conn_ids: Vec<Uuid>,
    service: &DatabaseService,
) -> Vec<Result<Metadata, AppError>> {
    // 使用 join_all 并发获取所有元数据
    let futures = conn_ids.iter().map(|id| service.get_metadata(id));
    futures::future::join_all(futures).await
}

// ✅ 好的实践：使用 select! 处理多个异步操作
use tokio::select;

pub async fn execute_with_timeout(
    query: String,
    adapter: Arc<dyn DatabaseAdapter>,
    timeout: Duration,
) -> Result<QueryResult, AppError> {
    select! {
        result = adapter.execute_query(&query) => result,
        _ = tokio::time::sleep(timeout) => {
            Err(AppError::QueryTimeout(format!("Query exceeded {}s", timeout.as_secs())))
        }
    }
}

// ✅ 好的实践：避免不必要的 .await
async fn sequential_operations() -> Result<(), AppError> {
    // 先启动所有异步操作
    let fetch1 = fetch_data(1);
    let fetch2 = fetch_data(2);
    let fetch3 = fetch_data(3);

    // 然后等待它们全部完成
    let (r1, r2, r3) = tokio::join!(fetch1, fetch2, fetch3);
    Ok(())
}

// ❌ 避免：串行执行可以并行的操作
async fn bad_sequential() {
    let r1 = fetch_data(1).await; // 等待
    let r2 = fetch_data(2).await; // 等待
    let r3 = fetch_data(3).await; // 等待
}
```

### 2.3 共享状态管理

**原则**: 使用 Arc<Mutex<T>> 或 Arc<RwLock<T>> 在任务间共享可变状态。

```rust
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

// ✅ 好的实践：RwLock 用于读多写少场景
#[derive(Clone)]
pub struct AppState {
    pub db_service: Arc<DatabaseService>,
    pub metadata_cache: Arc<RwLock<HashMap<Uuid, Metadata>>>,
    pub config: Arc<Config>, // 不可变，不需要锁
}

impl AppState {
    // 读取元数据（多个任务可同时读）
    pub async fn get_cached_metadata(&self, id: &Uuid) -> Option<Metadata> {
        let cache = self.metadata_cache.read().await;
        cache.get(id).cloned()
    }

    // 更新元数据（独占写）
    pub async fn update_cache(&self, id: Uuid, metadata: Metadata) {
        let mut cache = self.metadata_cache.write().await;
        cache.insert(id, metadata);
    }
}

// ✅ 好的实践：Mutex 用于读写频率相当场景
pub struct QueryExecutor {
    active_queries: Arc<Mutex<HashSet<Uuid>>>,
}

impl QueryExecutor {
    pub async fn start_query(&self, query_id: Uuid) -> bool {
        let mut queries = self.active_queries.lock().await;
        queries.insert(query_id)
    }

    pub async fn end_query(&self, query_id: &Uuid) {
        let mut queries = self.active_queries.lock().await;
        queries.remove(query_id);
    }
}

// ✅ 好的实践：最小化锁持有时间
pub async fn optimized_cache_update(
    cache: &Arc<RwLock<HashMap<Uuid, Metadata>>>,
    id: Uuid,
    metadata: Metadata,
) {
    // 不要在锁内执行异步操作
    {
        let mut cache = cache.write().await;
        cache.insert(id, metadata);
    } // 锁在这里释放

    // 在锁外执行其他操作
    notify_update(id).await;
}

// ❌ 避免：在锁内执行长时间异步操作
async fn bad_lock_holding(cache: &Arc<RwLock<HashMap<Uuid, Metadata>>>) {
    let mut cache = cache.write().await;
    // 持有写锁期间执行网络请求 - 阻塞所有读者！
    let data = fetch_from_network().await;
    cache.insert(Uuid::new_v4(), data);
}
```

### 2.4 通道 (Channels) 通信

**原则**: 使用通道在任务间传递消息，避免共享状态。

```rust
use tokio::sync::{mpsc, oneshot};

// ✅ 好的实践：使用 mpsc 通道进行任务通信
pub struct QueryProcessor {
    tx: mpsc::Sender<QueryRequest>,
}

pub struct QueryRequest {
    query: String,
    response: oneshot::Sender<Result<QueryResult, AppError>>,
}

impl QueryProcessor {
    pub fn new() -> (Self, mpsc::Receiver<QueryRequest>) {
        let (tx, rx) = mpsc::channel(100); // 缓冲区大小 100
        (Self { tx }, rx)
    }

    pub async fn submit_query(&self, query: String) -> Result<QueryResult, AppError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(QueryRequest {
                query,
                response: response_tx,
            })
            .await
            .map_err(|_| AppError::InternalError("Processor closed".to_string()))?;

        response_rx
            .await
            .map_err(|_| AppError::InternalError("Response channel closed".to_string()))?
    }
}

// 处理器任务
pub async fn run_query_processor(
    mut rx: mpsc::Receiver<QueryRequest>,
    adapter: Arc<dyn DatabaseAdapter>,
) {
    while let Some(req) = rx.recv().await {
        let result = adapter.execute_query(&req.query).await;
        let _ = req.response.send(result); // 忽略发送失败（客户端已断开）
    }
}

// ✅ 好的实践：使用 broadcast 通道进行多播
use tokio::sync::broadcast;

pub struct EventBus {
    tx: broadcast::Sender<Event>,
}

#[derive(Clone, Debug)]
pub enum Event {
    ConnectionCreated(Uuid),
    QueryExecuted(Uuid),
    MetadataRefreshed(Uuid),
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn publish(&self, event: Event) {
        let _ = self.tx.send(event); // 忽略无订阅者的情况
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}
```

### 2.5 避免常见异步陷阱

```rust
// ❌ 避免：无限制的任务生成
async fn bad_unbounded_spawning(queries: Vec<String>) {
    for query in queries {
        tokio::spawn(async move {
            execute_query(query).await;
        });
    }
    // 可能生成数千个任务，耗尽资源
}

// ✅ 好的实践：使用信号量限制并发
use tokio::sync::Semaphore;

pub async fn bounded_query_execution(
    queries: Vec<String>,
    max_concurrent: usize,
) -> Vec<Result<QueryResult, AppError>> {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let tasks: Vec<_> = queries
        .into_iter()
        .map(|query| {
            let permit = semaphore.clone();
            tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                execute_query(query).await
            })
        })
        .collect();

    futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect()
}

// ❌ 避免：Send trait 边界问题
async fn bad_rc_across_await() {
    let rc = Rc::new(5); // Rc 不是 Send
    some_async_fn().await;
    println!("{}", rc); // 编译错误：Rc 不能跨 await 点
}

// ✅ 好的实践：使用 Arc 代替 Rc
async fn good_arc_across_await() {
    let arc = Arc::new(5); // Arc 是 Send + Sync
    some_async_fn().await;
    println!("{}", arc); // OK
}

// ✅ 好的实践：正确处理取消安全性
use tokio::time::{sleep, Duration};

pub async fn cancellation_safe_operation(state: Arc<RwLock<State>>) {
    // 使用 select! 时确保操作是取消安全的
    tokio::select! {
        _ = sleep(Duration::from_secs(10)) => {
            // 超时分支
        }
        _ = process_data(state.clone()) => {
            // 处理分支 - 确保中途取消不会导致数据不一致
        }
    }
}
```

---

## 3. 错误处理

### 3.1 Result<T, E> 模式

**原则**: 使用 Result 处理可恢复错误，使用 panic! 处理不可恢复错误。

```rust
// ✅ 好的实践：定义清晰的错误类型
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Invalid SQL query: {0}")]
    InvalidSql(String),

    #[error("Query timeout after {0}s")]
    QueryTimeout(u64),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(Uuid),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

// ✅ 为 Axum 实现 IntoResponse
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::ConnectionFailed(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            AppError::InvalidSql(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::QueryTimeout(_) => (StatusCode::REQUEST_TIMEOUT, self.to_string()),
            AppError::ConnectionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };

        (status, message).into_response()
    }
}
```

### 3.2 错误传播与 ? 操作符

```rust
// ✅ 好的实践：使用 ? 简洁传播错误
pub async fn create_connection(
    config: ConnectionConfig,
) -> Result<DatabaseConnection, AppError> {
    // 验证配置
    validate_config(&config)?;

    // 建立连接 - sqlx::Error 自动转换为 AppError
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.url)
        .await?;

    // 测试连接
    sqlx::query("SELECT 1").fetch_one(&pool).await?;

    Ok(DatabaseConnection::new(pool))
}

// ✅ 好的实践：添加错误上下文
use anyhow::Context;

pub async fn load_metadata_from_cache(
    conn_id: &Uuid,
) -> Result<Metadata, anyhow::Error> {
    let path = format!("./cache/{}.json", conn_id);

    let content = tokio::fs::read_to_string(&path)
        .await
        .context(format!("Failed to read metadata file: {}", path))?;

    serde_json::from_str(&content)
        .context("Failed to parse metadata JSON")
}

// ✅ 好的实践：map_err 自定义错误转换
pub async fn execute_validated_query(
    adapter: &dyn DatabaseAdapter,
    sql: &str,
) -> Result<QueryResult, AppError> {
    // 验证 SQL
    let (validated_sql, _) = SqlValidator::validate_and_prepare(sql, 1000)
        .map_err(|e| AppError::InvalidSql(e.to_string()))?;

    // 执行查询
    adapter
        .execute_query(&validated_sql)
        .await
        .map_err(|e| AppError::DatabaseError(e))
}
```

### 3.3 自定义错误类型 (thiserror vs anyhow)

**原则**: 库使用 thiserror，应用使用 anyhow。

```rust
// ✅ thiserror：用于库代码，定义具体错误类型
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqlValidationError {
    #[error("Failed to parse SQL: {0}")]
    ParseError(String),

    #[error("Only SELECT queries are allowed, found: {0}")]
    ForbiddenStatement(String),

    #[error("Subqueries are not supported")]
    SubqueryNotSupported,

    #[error("Query contains potentially dangerous operation: {0}")]
    DangerousOperation(String),
}

// ✅ anyhow：用于应用代码，灵活的错误处理
use anyhow::{anyhow, Context, Result};

pub async fn bootstrap_application() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置
    let config = load_config()
        .context("Failed to load application configuration")?;

    // 初始化数据库
    let db = init_database(&config.database_url)
        .await
        .context("Failed to initialize database")?;

    // 启动服务器
    start_server(config, db)
        .await
        .context("Failed to start server")?;

    Ok(())
}

// ✅ 好的实践：错误包装与传播
pub async fn get_connection_metadata(
    storage: &SqliteStorage,
    conn_id: &Uuid,
) -> Result<Metadata, AppError> {
    match storage.get_metadata(conn_id).await {
        Ok(Some(metadata)) => Ok(metadata),
        Ok(None) => Err(AppError::ConnectionNotFound(*conn_id)),
        Err(e) => Err(AppError::DatabaseError(e)),
    }
}
```

### 3.4 何时使用 panic!

**原则**: panic! 用于编程错误，Result 用于可恢复错误。

```rust
// ✅ 好的实践：使用 expect 记录 panic 原因
pub fn new_with_capacity(capacity: usize) -> Self {
    assert!(capacity > 0, "Capacity must be greater than 0");

    Self {
        buffer: Vec::with_capacity(capacity),
        config: Config::default(),
    }
}

// ✅ 好的实践：在 main 函数中处理所有错误
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 所有错误都返回 Result，不要 panic
    let config = load_config()?;
    let app = create_app(config).await?;
    run_server(app).await?;
    Ok(())
}

// ✅ 好的实践：unwrap 仅用于明确不会失败的情况
pub fn get_default_port() -> u16 {
    std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid u16") // 配置错误，应该在启动时发现
}

// ❌ 避免：在库代码中 panic
pub fn bad_query_execution(sql: &str) -> QueryResult {
    if !sql.starts_with("SELECT") {
        panic!("Only SELECT queries allowed!"); // 应该返回 Result
    }
    // ...
}

// ✅ 正确：返回 Result
pub fn good_query_execution(sql: &str) -> Result<QueryResult, AppError> {
    if !sql.starts_with("SELECT") {
        return Err(AppError::InvalidSql("Only SELECT queries allowed".to_string()));
    }
    // ...
}
```

### 3.5 错误日志与监控

```rust
use tracing::{error, warn, info, debug};

// ✅ 好的实践：记录错误上下文
pub async fn execute_query_with_logging(
    conn_id: Uuid,
    query: &str,
) -> Result<QueryResult, AppError> {
    info!("Executing query for connection {}", conn_id);
    debug!("Query SQL: {}", query);

    match execute_query_internal(conn_id, query).await {
        Ok(result) => {
            info!("Query executed successfully, rows: {}", result.rows.len());
            Ok(result)
        }
        Err(e) => {
            error!(
                error = %e,
                conn_id = %conn_id,
                query = %query,
                "Query execution failed"
            );
            Err(e)
        }
    }
}

// ✅ 好的实践：使用 tracing span 跟踪请求
use tracing::{instrument, Span};

#[instrument(skip(adapter), fields(query_id = %Uuid::new_v4()))]
pub async fn execute_with_tracing(
    adapter: Arc<dyn DatabaseAdapter>,
    query: String,
) -> Result<QueryResult, AppError> {
    let span = Span::current();
    span.record("sql", &query.as_str());

    let result = adapter.execute_query(&query).await;

    match &result {
        Ok(r) => span.record("rows", r.rows.len()),
        Err(e) => span.record("error", &e.to_string()),
    }

    result
}
```

---

## 4. 类型系统与 API 设计

### 4.1 Trait 设计原则

**原则**: 小而聚焦的 trait，使用关联类型和默认实现。

```rust
// ✅ 好的实践：定义清晰的 trait 边界
use async_trait::async_trait;
use datafusion::arrow::record_batch::RecordBatch;

#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    /// 执行查询并返回结果
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, AppError>;

    /// 获取数据库元数据
    async fn get_metadata(&self) -> Result<Metadata, AppError>;

    /// 测试连接是否有效
    async fn test_connection(&self) -> Result<bool, AppError> {
        // 默认实现
        match self.execute_query("SELECT 1").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 关闭连接
    async fn close(&self) -> Result<(), AppError>;
}

// ✅ 实现具体的适配器
pub struct PostgresAdapter {
    pool: sqlx::PgPool,
    dialect: PostgreSqlDialect,
}

#[async_trait]
impl DatabaseAdapter for PostgresAdapter {
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, AppError> {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await?;

        Ok(QueryResult::from_sqlx_rows(rows))
    }

    async fn get_metadata(&self) -> Result<Metadata, AppError> {
        // PostgreSQL 特定的元数据查询
        let tables = sqlx::query_as::<_, TableInfo>(
            "SELECT table_name, table_schema FROM information_schema.tables"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(Metadata { tables })
    }

    async fn close(&self) -> Result<(), AppError> {
        self.pool.close().await;
        Ok(())
    }
}

// ✅ 好的实践：trait 组合
pub trait QueryExecutor: DatabaseAdapter {
    fn validate_sql(&self, sql: &str) -> Result<String, SqlValidationError>;
}

// ✅ 好的实践：关联类型
pub trait DataSerializer {
    type Output;
    type Error;

    fn serialize(&self, data: &QueryResult) -> Result<Self::Output, Self::Error>;
}

pub struct JsonSerializer;

impl DataSerializer for JsonSerializer {
    type Output = String;
    type Error = serde_json::Error;

    fn serialize(&self, data: &QueryResult) -> Result<String, serde_json::Error> {
        serde_json::to_string(data)
    }
}
```

### 4.2 泛型编程

**原则**: 使用泛型提高代码复用性，但避免过度泛型化。

```rust
// ✅ 好的实践：泛型函数
pub async fn execute_with_retry<F, Fut, T, E>(
    mut operation: F,
    max_retries: usize,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempts = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                warn!("Operation failed (attempt {}/{}): {}", attempts, max_retries, e);
                tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

// 使用示例
let result = execute_with_retry(
    || async { fetch_metadata(&conn).await },
    3
).await?;

// ✅ 好的实践：泛型结构体
pub struct CachedService<T, S>
where
    S: Service<T>,
{
    service: S,
    cache: Arc<RwLock<HashMap<String, T>>>,
    ttl: Duration,
}

impl<T, S> CachedService<T, S>
where
    T: Clone + Send + Sync + 'static,
    S: Service<T> + Send + Sync,
{
    pub async fn get(&self, key: &str) -> Result<T, S::Error> {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(value) = cache.get(key) {
                return Ok(value.clone());
            }
        }

        // 缓存未命中，从服务获取
        let value = self.service.fetch(key).await?;

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), value.clone());
        }

        Ok(value)
    }
}

// ✅ 好的实践：impl Trait 简化返回类型
pub fn filter_valid_queries(queries: Vec<String>) -> impl Iterator<Item = String> {
    queries
        .into_iter()
        .filter(|q| q.starts_with("SELECT"))
        .map(|q| q.trim().to_string())
}

// ❌ 避免：过度泛型化导致难以理解
pub fn bad_generic<T, U, V, F, G, H>(
    x: T,
    f: F,
    g: G,
    h: H,
) -> Result<V, Box<dyn std::error::Error>>
where
    T: Clone + Send + Sync,
    U: From<T>,
    V: From<U>,
    F: Fn(T) -> U,
    G: Fn(U) -> V,
    H: Fn(V) -> Result<V, Box<dyn std::error::Error>>,
{
    // 太复杂了！
    h(g(f(x)))
}
```

### 4.3 Builder 模式

**原则**: 为复杂结构提供流畅的构造接口。

```rust
// ✅ 好的实践：Builder 模式
pub struct ConnectionConfig {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    max_connections: u32,
    timeout: Duration,
    ssl_mode: SslMode,
}

pub struct ConnectionConfigBuilder {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: Option<String>,
    max_connections: u32,
    timeout: Duration,
    ssl_mode: SslMode,
}

impl ConnectionConfigBuilder {
    pub fn new(host: impl Into<String>, database: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: 5432,
            database: database.into(),
            username: "postgres".to_string(),
            password: None,
            max_connections: 10,
            timeout: Duration::from_secs(30),
            ssl_mode: SslMode::Prefer,
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn ssl_mode(mut self, mode: SslMode) -> Self {
        self.ssl_mode = mode;
        self
    }

    pub fn build(self) -> Result<ConnectionConfig, ConfigError> {
        if self.password.is_none() {
            return Err(ConfigError::MissingPassword);
        }

        Ok(ConnectionConfig {
            host: self.host,
            port: self.port,
            database: self.database,
            username: self.username,
            password: self.password.unwrap(),
            max_connections: self.max_connections,
            timeout: self.timeout,
            ssl_mode: self.ssl_mode,
        })
    }
}

// 使用示例
let config = ConnectionConfigBuilder::new("localhost", "mydb")
    .port(5432)
    .username("admin")
    .password("secret")
    .max_connections(20)
    .timeout(Duration::from_secs(60))
    .build()?;
```

### 4.4 Newtype 模式提高类型安全

**原则**: 使用 newtype 包装原始类型，防止混淆。

```rust
// ✅ 好的实践：Newtype 模式
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryId(Uuid);

// 现在不会意外混淆 ConnectionId 和 QueryId
pub fn get_connection(id: ConnectionId) -> Option<Connection> {
    // 编译器确保传入正确的类型
    todo!()
}

// ❌ 避免：使用原始类型容易混淆
pub fn bad_get_connection(id: Uuid) -> Option<Connection> {
    // 无法区分这是 ConnectionId 还是 QueryId
    todo!()
}

// ✅ 好的实践：为业务规则创建类型
#[derive(Debug, Clone)]
pub struct Limit(u32);

impl Limit {
    pub const MAX: u32 = 10000;
    pub const DEFAULT: u32 = 1000;

    pub fn new(value: u32) -> Result<Self, ValidationError> {
        if value == 0 {
            return Err(ValidationError::LimitTooSmall);
        }
        if value > Self::MAX {
            return Err(ValidationError::LimitTooLarge(Self::MAX));
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for Limit {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

// 使用
pub fn execute_query(sql: &str, limit: Limit) -> Result<QueryResult, AppError> {
    // limit 保证有效，不需要再次验证
    let sql_with_limit = format!("{} LIMIT {}", sql, limit.value());
    // ...
}
```

### 4.5 Phantom Types

**原则**: 使用幻象类型在编译时强制状态机转换。

```rust
// ✅ 好的实践：类型状态模式
use std::marker::PhantomData;

pub struct Unvalidated;
pub struct Validated;

pub struct SqlQuery<State = Unvalidated> {
    sql: String,
    _state: PhantomData<State>,
}

impl SqlQuery<Unvalidated> {
    pub fn new(sql: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            _state: PhantomData,
        }
    }

    pub fn validate(self) -> Result<SqlQuery<Validated>, SqlValidationError> {
        // 验证逻辑
        let validated = SqlValidator::validate(&self.sql)?;

        Ok(SqlQuery {
            sql: validated,
            _state: PhantomData,
        })
    }
}

impl SqlQuery<Validated> {
    pub fn execute(&self, adapter: &dyn DatabaseAdapter) -> Result<QueryResult, AppError> {
        // 只有验证过的查询才能执行
        adapter.execute_query(&self.sql)
    }

    pub fn sql(&self) -> &str {
        &self.sql
    }
}

// 使用示例
let query = SqlQuery::new("SELECT * FROM users");
// query.execute(&adapter); // 编译错误！未验证的查询不能执行

let validated = query.validate()?;
let result = validated.execute(&adapter)?; // OK
```

---

## 5. 性能与优化

### 5.1 避免不必要的克隆

**原则**: 优先使用引用，仅在必要时克隆。

```rust
// ✅ 好的实践：使用引用
pub fn format_table_names(metadata: &Metadata) -> Vec<String> {
    metadata.tables
        .iter()
        .map(|table| format!("{}.{}", table.schema, table.name))
        .collect()
}

// ❌ 避免：不必要的克隆
pub fn bad_format_table_names(metadata: Metadata) -> Vec<String> {
    metadata.tables.clone() // 不必要的克隆
        .iter()
        .map(|table| format!("{}.{}", table.schema, table.name))
        .collect()
}

// ✅ 好的实践：Cow (Clone on Write)
use std::borrow::Cow;

pub fn normalize_sql(sql: &str) -> Cow<str> {
    if sql.contains('\t') || sql.contains('\n') {
        // 需要修改，返回 owned
        Cow::Owned(sql.replace('\t', " ").replace('\n', " "))
    } else {
        // 不需要修改，返回 borrowed
        Cow::Borrowed(sql)
    }
}

// ✅ 好的实践：使用 Arc 在线程间共享
#[derive(Clone)]
pub struct QueryCache {
    data: Arc<RwLock<HashMap<String, QueryResult>>>,
}

impl QueryCache {
    pub async fn get(&self, key: &str) -> Option<QueryResult> {
        let cache = self.data.read().await;
        cache.get(key).cloned() // 只克隆 QueryResult，不克隆整个 HashMap
    }
}

// ✅ 好的实践：返回引用而非所有权
pub struct Metadata {
    tables: Vec<Table>,
}

impl Metadata {
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        // 返回引用，避免克隆
        self.tables.iter().find(|t| t.name == name)
    }

    pub fn table_names(&self) -> impl Iterator<Item = &str> {
        // 返回迭代器，零拷贝
        self.tables.iter().map(|t| t.name.as_str())
    }
}
```

### 5.2 迭代器链 vs 显式循环

**原则**: 优先使用迭代器链，除非需要提前返回或复杂控制流。

```rust
// ✅ 好的实践：迭代器链（编译器优化更好）
pub fn filter_and_transform_tables(metadata: &Metadata) -> Vec<String> {
    metadata.tables
        .iter()
        .filter(|t| t.row_count > 1000)
        .filter(|t| !t.name.starts_with("_"))
        .map(|t| t.name.to_uppercase())
        .collect()
}

// ✅ 好的实践：使用 filter_map 组合过滤和映射
pub fn extract_column_types(metadata: &Metadata, table_name: &str) -> Vec<String> {
    metadata.tables
        .iter()
        .find(|t| t.name == table_name)
        .map(|table| {
            table.columns
                .iter()
                .map(|col| col.data_type.clone())
                .collect()
        })
        .unwrap_or_default()
}

// ✅ 好的实践：parallel iterators (CPU 密集型)
use rayon::prelude::*;

pub fn process_large_result_set(rows: Vec<Row>) -> Vec<ProcessedRow> {
    rows.par_iter() // 并行处理
        .map(|row| process_expensive_operation(row))
        .collect()
}

// ❌ 避免：手动循环分配 Vec
pub fn bad_collect() -> Vec<String> {
    let mut result = Vec::new();
    for i in 0..100 {
        result.push(format!("item_{}", i));
    }
    result
}

// ✅ 好的实践：预分配容量
pub fn good_collect() -> Vec<String> {
    (0..100)
        .map(|i| format!("item_{}", i))
        .collect() // 编译器优化会预分配
}

// ✅ 当需要提前返回时使用显式循环
pub fn find_first_valid_connection(configs: &[ConnectionConfig]) -> Option<Connection> {
    for config in configs {
        if let Ok(conn) = connect(config) {
            return Some(conn); // 提前返回
        }
    }
    None
}
```

### 5.3 零拷贝技术

**原则**: 使用 Bytes、引用和切片避免内存分配。

```rust
use bytes::Bytes;

// ✅ 好的实践：使用 Bytes 进行零拷贝
pub struct Response {
    body: Bytes, // Arc 内部，clone 只复制指针
}

impl Response {
    pub fn from_string(s: String) -> Self {
        Self {
            body: Bytes::from(s),
        }
    }

    pub fn clone_body(&self) -> Bytes {
        self.body.clone() // 只复制指针，不复制数据
    }
}

// ✅ 好的实践：字符串切片避免分配
pub fn extract_table_name(full_name: &str) -> &str {
    full_name
        .split('.')
        .last()
        .unwrap_or(full_name)
}

// ✅ 好的实践：使用 AsRef 接受多种类型
pub fn execute_sql<S: AsRef<str>>(sql: S) -> Result<QueryResult, AppError> {
    let sql = sql.as_ref();
    // 可以接受 &str, String, Cow<str> 等
    validate_and_execute(sql)
}

// ✅ 好的实践：使用 serde 的 borrow 避免字符串拷贝
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryRequest<'a> {
    #[serde(borrow)]
    pub sql: Cow<'a, str>,
    pub connection_id: Uuid,
}
```

### 5.4 性能分析与基准测试

**原则**: 测量再优化，不要猜测瓶颈。

```rust
// ✅ 使用 criterion 进行基准测试
#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn benchmark_sql_validation(c: &mut Criterion) {
        let sql = "SELECT * FROM users WHERE id = 1";

        c.bench_function("validate_sql", |b| {
            b.iter(|| {
                SqlValidator::validate_and_prepare(black_box(sql), 1000)
            })
        });
    }

    criterion_group!(benches, benchmark_sql_validation);
    criterion_main!(benches);
}

// ✅ 使用 tracing 进行性能分析
use tracing::{instrument, debug};

#[instrument(skip(adapter))]
pub async fn execute_with_timing(
    adapter: &dyn DatabaseAdapter,
    sql: &str,
) -> Result<QueryResult, AppError> {
    let start = std::time::Instant::now();

    let result = adapter.execute_query(sql).await?;

    let duration = start.elapsed();
    debug!("Query executed in {:?}", duration);

    Ok(result)
}

// ✅ 使用 cargo flamegraph 生成火焰图
// 在 Cargo.toml 中添加 [profile.release] 配置
// [profile.release]
// debug = true
//
// 运行：cargo flamegraph --bin db-query-backend
```

### 5.5 内存分配优化

```rust
// ✅ 好的实践：预分配容量
pub fn collect_table_names(metadata: &Metadata) -> Vec<String> {
    let mut names = Vec::with_capacity(metadata.tables.len());
    for table in &metadata.tables {
        names.push(table.name.clone());
    }
    names
}

// ✅ 好的实践：复用缓冲区
pub struct QueryProcessor {
    buffer: Vec<u8>,
}

impl QueryProcessor {
    pub fn process(&mut self, data: &[u8]) -> Result<String, Error> {
        self.buffer.clear(); // 复用而非重新分配
        self.buffer.extend_from_slice(data);
        // 处理数据
        Ok(String::from_utf8_lossy(&self.buffer).to_string())
    }
}

// ✅ 好的实践：使用 SmallVec 优化小集合
use smallvec::SmallVec;

pub fn get_primary_keys(table: &Table) -> SmallVec<[String; 4]> {
    // 大多数表有 1-4 个主键，避免堆分配
    table.columns
        .iter()
        .filter(|c| c.is_primary_key)
        .map(|c| c.name.clone())
        .collect()
}

// ✅ 好的实践：使用 Box<[T]> 代替 Vec<T> 节省内存
pub fn freeze_table_list(tables: Vec<Table>) -> Box<[Table]> {
    // 不再需要增长容量，释放额外空间
    tables.into_boxed_slice()
}
```

---

## 6. Web 服务开发 (Axum/Tokio)

### 6.1 请求处理器模式

**原则**: 清晰的路由定义，处理器职责单一。

```rust
use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Path, Query, State},
    Json,
};

// ✅ 好的实践：清晰的路由定义
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/connections", post(create_connection))
        .route("/api/connections/:id", get(get_connection))
        .route("/api/connections/:id", delete(delete_connection))
        .route("/api/connections/:id/metadata", get(get_metadata))
        .route("/api/query", post(execute_query))
        .route("/api/query/natural", post(execute_natural_language_query))
        .with_state(state)
}

// ✅ 好的实践：类型安全的处理器
pub async fn create_connection(
    State(state): State<AppState>,
    Json(payload): Json<CreateConnectionRequest>,
) -> Result<Json<ConnectionResponse>, AppError> {
    // 验证输入
    payload.validate()?;

    // 创建连接
    let conn = state
        .db_service
        .create_connection(payload.into())
        .await?;

    // 返回响应
    Ok(Json(ConnectionResponse::from(conn)))
}

// ✅ 好的实践：使用 Path 提取路径参数
pub async fn get_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConnectionResponse>, AppError> {
    let conn = state
        .db_service
        .get_connection(&id)
        .await
        .ok_or(AppError::ConnectionNotFound(id))?;

    Ok(Json(ConnectionResponse::from(conn)))
}

// ✅ 好的实践：使用 Query 提取查询参数
#[derive(Deserialize)]
pub struct MetadataParams {
    #[serde(default)]
    refresh: bool,
}

pub async fn get_metadata(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<MetadataParams>,
) -> Result<Json<Metadata>, AppError> {
    if params.refresh {
        state.metadata_cache.invalidate(&id).await;
    }

    let metadata = state
        .metadata_cache
        .get_or_fetch(&id)
        .await?;

    Ok(Json(metadata))
}
```

### 6.2 状态管理

**原则**: 使用 Arc 共享状态，避免克隆大对象。

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

// ✅ 好的实践：应用状态设计
#[derive(Clone)]
pub struct AppState {
    pub db_service: Arc<DatabaseService>,
    pub metadata_cache: Arc<MetadataCache>,
    pub query_executor: Arc<QueryExecutor>,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let db_service = Arc::new(DatabaseService::new());
        let metadata_cache = Arc::new(MetadataCache::new(db_service.clone()));
        let query_executor = Arc::new(QueryExecutor::new(db_service.clone()));

        Self {
            db_service,
            metadata_cache,
            query_executor,
            config: Arc::new(config),
        }
    }
}

// ✅ 好的实践：服务层封装业务逻辑
pub struct DatabaseService {
    connections: Arc<RwLock<HashMap<Uuid, Arc<DatabaseConnection>>>>,
    storage: Arc<SqliteStorage>,
}

impl DatabaseService {
    pub async fn create_connection(
        &self,
        config: ConnectionConfig,
    ) -> Result<DatabaseConnection, AppError> {
        // 建立连接
        let connection = DatabaseConnection::connect(config).await?;

        // 保存到存储
        self.storage.save_connection(&connection).await?;

        // 缓存连接
        let id = connection.id;
        let mut connections = self.connections.write().await;
        connections.insert(id, Arc::new(connection.clone()));

        Ok(connection)
    }

    pub async fn get_connection(&self, id: &Uuid) -> Option<Arc<DatabaseConnection>> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }
}

// ✅ 好的实践：使用 Extension 传递请求级数据
use axum::extract::Extension;

pub struct RequestId(pub Uuid);

// 中间件添加 request_id
pub async fn add_request_id<B>(
    mut req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> axum::response::Response {
    let request_id = RequestId(Uuid::new_v4());
    req.extensions_mut().insert(request_id);
    next.run(req).await
}

// 处理器中使用
pub async fn handler(Extension(request_id): Extension<RequestId>) {
    tracing::info!("Processing request {}", request_id.0);
}
```

### 6.3 中间件设计

**原则**: 中间件用于横切关注点（日志、认证、错误处理）。

```rust
use axum::{
    middleware::{self, Next},
    http::{Request, StatusCode},
    response::{Response, IntoResponse},
};
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
};

// ✅ 好的实践：组合中间件
pub fn create_router_with_middleware(state: AppState) -> Router {
    Router::new()
        .route("/api/query", post(execute_query))
        // 应用级中间件
        .layer(middleware::from_fn(add_request_id))
        .layer(middleware::from_fn(log_request))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ✅ 好的实践：请求日志中间件
pub async fn log_request<B>(
    req: Request<B>,
    next: Next<B>,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    tracing::info!(
        method = %method,
        uri = %uri,
        status = %status,
        duration = ?duration,
        "Request processed"
    );

    response
}

// ✅ 好的实践：错误处理中间件
pub async fn handle_errors<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let response = next.run(req).await;

    if response.status().is_server_error() {
        tracing::error!("Server error: {:?}", response);
    }

    Ok(response)
}

// ✅ 好的实践：认证中间件
pub async fn require_auth<B>(
    Extension(user): Extension<Option<User>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    match user {
        Some(_) => Ok(next.run(req).await),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
```

### 6.4 请求验证与反序列化

**原则**: 在边界验证数据，使用类型系统保证内部有效性。

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

// ✅ 好的实践：请求验证
#[derive(Debug, Deserialize, Validate)]
pub struct CreateConnectionRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(url)]
    pub url: String,

    #[validate(range(min = 1, max = 100))]
    pub max_connections: u32,
}

impl CreateConnectionRequest {
    pub fn validate(&self) -> Result<(), AppError> {
        Validate::validate(self)
            .map_err(|e| AppError::ValidationError(e.to_string()))
    }
}

// ✅ 好的实践：分离请求/响应/领域模型
#[derive(Serialize)]
pub struct ConnectionResponse {
    pub id: Uuid,
    pub name: String,
    pub database_type: String,
    pub created_at: i64,
}

impl From<DatabaseConnection> for ConnectionResponse {
    fn from(conn: DatabaseConnection) -> Self {
        Self {
            id: conn.id,
            name: conn.name,
            database_type: conn.database_type.to_string(),
            created_at: conn.created_at.timestamp(),
        }
    }
}

// ✅ 好的实践：自定义反序列化器
use serde::de::{self, Deserializer};

pub fn deserialize_limit<'de, D>(deserializer: D) -> Result<Limit, D::Error>
where
    D: Deserializer<'de>,
{
    let value: u32 = Deserialize::deserialize(deserializer)?;
    Limit::new(value).map_err(de::Error::custom)
}

#[derive(Deserialize)]
pub struct QueryRequest {
    pub sql: String,

    #[serde(deserialize_with = "deserialize_limit")]
    pub limit: Limit,
}
```

### 6.5 响应构建

**原则**: 类型安全的响应构建，清晰的状态码。

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

// ✅ 好的实践：自定义响应类型
pub struct ApiResponse<T> {
    pub data: T,
    pub message: Option<String>,
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

// ✅ 好的实践：不同状态码的响应
pub async fn create_connection(
    State(state): State<AppState>,
    Json(req): Json<CreateConnectionRequest>,
) -> Result<impl IntoResponse, AppError> {
    req.validate()?;

    let conn = state.db_service.create_connection(req.into()).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse {
            data: ConnectionResponse::from(conn),
            message: Some("Connection created successfully".to_string()),
        }),
    ))
}

// ✅ 好的实践：流式响应
use axum::response::sse::{Event, Sse};
use tokio_stream::StreamExt;

pub async fn stream_query_results(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let stream = state
        .query_executor
        .execute_streaming(&req.sql)
        .await?
        .map(|row| {
            Ok(Event::default()
                .json_data(row)
                .unwrap())
        });

    Ok(Sse::new(stream))
}
```

---

## 7. 数据库集成

### 7.1 DataFusion 使用模式

**原则**: 利用 DataFusion 作为语义层，统一多数据源查询。

```rust
use datafusion::prelude::*;
use datafusion::arrow::record_batch::RecordBatch;

// ✅ 好的实践：DataFusion 执行上下文
pub struct DataFusionAdapter {
    ctx: SessionContext,
    connection_url: String,
}

impl DataFusionAdapter {
    pub async fn new(connection_url: String) -> Result<Self, AppError> {
        let ctx = SessionContext::new();

        // 注册数据源
        // 示例：注册 Parquet 文件
        // ctx.register_parquet("my_table", "path/to/file.parquet", None).await?;

        Ok(Self {
            ctx,
            connection_url,
        })
    }

    pub async fn execute_query(&self, sql: &str) -> Result<Vec<RecordBatch>, AppError> {
        let df = self.ctx
            .sql(sql)
            .await
            .map_err(|e| AppError::QueryError(e.to_string()))?;

        let batches = df
            .collect()
            .await
            .map_err(|e| AppError::QueryError(e.to_string()))?;

        Ok(batches)
    }

    pub async fn register_table(
        &self,
        table_name: &str,
        batches: Vec<RecordBatch>,
    ) -> Result<(), AppError> {
        let schema = batches[0].schema();
        let provider = datafusion::datasource::MemTable::try_new(schema, vec![batches])
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        self.ctx.register_table(table_name, Arc::new(provider))
            .map_err(|e| AppError::InternalError(e.to_string()))?;

        Ok(())
    }
}

// ✅ 好的实践：Arrow 数据转换
use datafusion::arrow::array::{Array, StringArray, Int64Array};

pub fn record_batch_to_json(batch: &RecordBatch) -> Result<Vec<serde_json::Value>, AppError> {
    let schema = batch.schema();
    let mut rows = Vec::with_capacity(batch.num_rows());

    for row_idx in 0..batch.num_rows() {
        let mut row = serde_json::Map::new();

        for (col_idx, field) in schema.fields().iter().enumerate() {
            let column = batch.column(col_idx);
            let value = match column.data_type() {
                datafusion::arrow::datatypes::DataType::Utf8 => {
                    let array = column.as_any().downcast_ref::<StringArray>().unwrap();
                    if array.is_null(row_idx) {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(array.value(row_idx).to_string())
                    }
                }
                datafusion::arrow::datatypes::DataType::Int64 => {
                    let array = column.as_any().downcast_ref::<Int64Array>().unwrap();
                    if array.is_null(row_idx) {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::Number(array.value(row_idx).into())
                    }
                }
                _ => serde_json::Value::Null,
            };

            row.insert(field.name().clone(), value);
        }

        rows.push(serde_json::Value::Object(row));
    }

    Ok(rows)
}
```

### 7.2 连接池管理

**原则**: 使用连接池，配置合理的超时和大小限制。

```rust
use sqlx::{postgres::PgPoolOptions, PgPool};

// ✅ 好的实践：连接池配置
pub struct DatabasePool {
    pool: PgPool,
}

impl DatabasePool {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(database_url)
            .await
            .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;

        // 测试连接
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| AppError::ConnectionFailed(e.to_string()))?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}

// ✅ 好的实践：健康检查
impl DatabasePool {
    pub async fn health_check(&self) -> bool {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .is_ok()
    }

    pub async fn stats(&self) -> PoolStats {
        PoolStats {
            connections: self.pool.size() as u32,
            idle: self.pool.num_idle(),
        }
    }
}

#[derive(Serialize)]
pub struct PoolStats {
    pub connections: u32,
    pub idle: usize,
}
```

### 7.3 查询执行策略

**原则**: 超时控制、结果限制、流式处理大结果集。

```rust
use sqlx::{Row, PgPool};
use futures::TryStreamExt;

// ✅ 好的实践：超时控制
pub async fn execute_with_timeout(
    pool: &PgPool,
    sql: &str,
    timeout: Duration,
) -> Result<QueryResult, AppError> {
    tokio::time::timeout(timeout, execute_query(pool, sql))
        .await
        .map_err(|_| AppError::QueryTimeout(timeout.as_secs()))?
}

// ✅ 好的实践：流式处理大结果集
pub async fn execute_streaming(
    pool: &PgPool,
    sql: &str,
) -> Result<impl Stream<Item = Result<Row, sqlx::Error>>, AppError> {
    let mut rows = sqlx::query(sql)
        .fetch(pool);

    Ok(rows)
}

// ✅ 好的实践：批量获取
pub async fn execute_batched(
    pool: &PgPool,
    sql: &str,
    batch_size: usize,
) -> Result<Vec<QueryResult>, AppError> {
    let mut stream = sqlx::query(sql).fetch(pool);
    let mut batches = Vec::new();
    let mut current_batch = Vec::with_capacity(batch_size);

    while let Some(row) = stream.try_next().await? {
        current_batch.push(row);

        if current_batch.len() >= batch_size {
            batches.push(QueryResult::from_rows(current_batch));
            current_batch = Vec::with_capacity(batch_size);
        }
    }

    if !current_batch.is_empty() {
        batches.push(QueryResult::from_rows(current_batch));
    }

    Ok(batches)
}

// ✅ 好的实践：准备语句防止 SQL 注入
pub async fn execute_parameterized(
    pool: &PgPool,
    table: &str,
    user_id: i64,
) -> Result<Vec<User>, AppError> {
    // 不要拼接 SQL！
    let sql = format!("SELECT * FROM {} WHERE user_id = $1", table);

    let users = sqlx::query_as::<_, User>(&sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?;

    Ok(users)
}
```

### 7.4 类型映射

**原则**: 明确数据库类型与 Rust 类型的映射。

```rust
use sqlx::FromRow;
use chrono::{DateTime, Utc};

// ✅ 好的实践：使用 FromRow 自动映射
#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
    pub metadata: Option<serde_json::Value>, // JSONB
}

// ✅ 好的实践：自定义类型映射
use sqlx::postgres::PgRow;
use sqlx::Row;

#[derive(Debug, Serialize)]
pub struct CustomUser {
    pub id: UserId,
    pub username: Username,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, PgRow> for CustomUser {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: UserId(row.try_get("id")?),
            username: Username::new(row.try_get("username")?),
            created_at: row.try_get("created_at")?,
        })
    }
}

// ✅ 好的实践：枚举类型映射
#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "database_type", rename_all = "lowercase")]
pub enum DatabaseType {
    Postgresql,
    Mysql,
    Doris,
    Druid,
}

// ✅ 好的实践：处理 NULL 值
pub async fn get_optional_column(
    pool: &PgPool,
    id: i64,
) -> Result<Option<String>, AppError> {
    let row = sqlx::query("SELECT optional_column FROM table WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row.and_then(|r| r.get("optional_column")))
}
```

### 7.5 事务管理

**原则**: 显式事务边界，正确处理提交和回滚。

```rust
use sqlx::Transaction;

// ✅ 好的实践：事务管理
pub async fn create_connection_with_metadata(
    pool: &PgPool,
    conn_config: ConnectionConfig,
    metadata: Metadata,
) -> Result<Uuid, AppError> {
    let mut tx = pool.begin().await?;

    // 插入连接
    let conn_id: Uuid = sqlx::query_scalar(
        "INSERT INTO connections (name, url, database_type) VALUES ($1, $2, $3) RETURNING id"
    )
    .bind(&conn_config.name)
    .bind(&conn_config.url)
    .bind(&conn_config.database_type)
    .fetch_one(&mut *tx)
    .await?;

    // 插入元数据
    sqlx::query(
        "INSERT INTO metadata (connection_id, data) VALUES ($1, $2)"
    )
    .bind(conn_id)
    .bind(serde_json::to_value(&metadata)?)
    .execute(&mut *tx)
    .await?;

    // 提交事务
    tx.commit().await?;

    Ok(conn_id)
}

// ✅ 好的实践：显式回滚
pub async fn update_with_rollback(
    pool: &PgPool,
    id: Uuid,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    match perform_update(&mut tx, id).await {
        Ok(_) => {
            tx.commit().await?;
            Ok(())
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

// ✅ 好的实践：嵌套事务（保存点）
pub async fn nested_transaction(tx: &mut Transaction<'_, sqlx::Postgres>) -> Result<(), AppError> {
    sqlx::query("SAVEPOINT my_savepoint")
        .execute(&mut **tx)
        .await?;

    match risky_operation(tx).await {
        Ok(_) => {
            sqlx::query("RELEASE SAVEPOINT my_savepoint")
                .execute(&mut **tx)
                .await?;
            Ok(())
        }
        Err(e) => {
            sqlx::query("ROLLBACK TO SAVEPOINT my_savepoint")
                .execute(&mut **tx)
                .await?;
            Err(e)
        }
    }
}
```

---

## 8. 测试与文档

### 8.1 单元测试组织

**原则**: 每个模块有对应的测试模块，测试函数命名清晰。

```rust
// ✅ 好的实践：测试模块组织
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_validator_accepts_select() {
        let sql = "SELECT * FROM users";
        let result = SqlValidator::validate_and_prepare(sql, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sql_validator_rejects_insert() {
        let sql = "INSERT INTO users (name) VALUES ('test')";
        let result = SqlValidator::validate_and_prepare(sql, 1000);
        assert!(matches!(result, Err(SqlValidationError::ForbiddenStatement(_))));
    }

    #[test]
    fn test_sql_validator_adds_limit() {
        let sql = "SELECT * FROM users";
        let (validated, limit_applied) = SqlValidator::validate_and_prepare(sql, 1000).unwrap();
        assert!(limit_applied);
        assert!(validated.contains("LIMIT 1000"));
    }

    #[test]
    fn test_limit_validation_rejects_zero() {
        let result = Limit::new(0);
        assert!(matches!(result, Err(ValidationError::LimitTooSmall)));
    }

    #[test]
    fn test_limit_validation_rejects_too_large() {
        let result = Limit::new(100_000);
        assert!(matches!(result, Err(ValidationError::LimitTooLarge(_))));
    }
}

// ✅ 好的实践：异步测试
#[cfg(test)]
mod async_tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let config = ConnectionConfig::test_config();
        let result = DatabaseConnection::connect(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_execution_with_timeout() {
        let adapter = MockAdapter::new();
        let result = execute_with_timeout(
            adapter,
            "SELECT * FROM large_table",
            Duration::from_secs(1),
        )
        .await;
        assert!(result.is_ok());
    }
}
```

### 8.2 集成测试

**原则**: 在 tests/ 目录编写集成测试，测试公共 API。

```rust
// tests/integration_test.rs

use db_query_backend::{create_app, AppState, Config};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check_endpoint() {
    let config = Config::test();
    let state = AppState::new(config);
    let app = create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_connection_endpoint() {
    let config = Config::test();
    let state = AppState::new(config);
    let app = create_app(state);

    let payload = serde_json::json!({
        "name": "test_db",
        "url": "postgresql://localhost/test",
        "max_connections": 10
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/connections")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

// ✅ 好的实践：使用测试 fixture
mod fixtures {
    use super::*;

    pub fn test_connection_config() -> ConnectionConfig {
        ConnectionConfig {
            name: "test".to_string(),
            url: "postgresql://localhost/test".to_string(),
            max_connections: 5,
            timeout: Duration::from_secs(30),
        }
    }

    pub async fn create_test_database() -> PgPool {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect("postgresql://localhost/test")
            .await
            .unwrap();

        // 清理并初始化测试数据
        sqlx::query("TRUNCATE TABLE connections CASCADE")
            .execute(&pool)
            .await
            .unwrap();

        pool
    }
}
```

### 8.3 Mock 和测试替身

**原则**: 使用 trait 设计使得 mock 更容易。

```rust
// ✅ 好的实践：定义 trait 便于 mock
#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, AppError>;
    async fn get_metadata(&self) -> Result<Metadata, AppError>;
}

// ✅ Mock 实现
pub struct MockDatabaseAdapter {
    pub query_results: Arc<Mutex<Vec<QueryResult>>>,
    pub metadata: Arc<Mutex<Option<Metadata>>>,
}

#[async_trait]
impl DatabaseAdapter for MockDatabaseAdapter {
    async fn execute_query(&self, _sql: &str) -> Result<QueryResult, AppError> {
        let mut results = self.query_results.lock().await;
        results.pop().ok_or(AppError::InternalError("No mock result".to_string()))
    }

    async fn get_metadata(&self) -> Result<Metadata, AppError> {
        let metadata = self.metadata.lock().await;
        metadata.clone().ok_or(AppError::InternalError("No mock metadata".to_string()))
    }
}

// ✅ 测试中使用 mock
#[tokio::test]
async fn test_query_service_with_mock() {
    let mock_adapter = MockDatabaseAdapter {
        query_results: Arc::new(Mutex::new(vec![
            QueryResult {
                rows: vec![],
                columns: vec![],
            },
        ])),
        metadata: Arc::new(Mutex::new(None)),
    };

    let result = mock_adapter.execute_query("SELECT 1").await;
    assert!(result.is_ok());
}

// ✅ 好的实践：使用 mockall crate
#[cfg(test)]
use mockall::{automock, predicate::*};

#[automock]
#[async_trait]
pub trait MetadataService: Send + Sync {
    async fn get_metadata(&self, conn_id: &Uuid) -> Result<Metadata, AppError>;
    async fn refresh_metadata(&self, conn_id: &Uuid) -> Result<Metadata, AppError>;
}

#[tokio::test]
async fn test_with_mockall() {
    let mut mock = MockMetadataService::new();

    mock.expect_get_metadata()
        .with(eq(Uuid::nil()))
        .times(1)
        .returning(|_| Ok(Metadata::default()));

    let result = mock.get_metadata(&Uuid::nil()).await;
    assert!(result.is_ok());
}
```

### 8.4 文档注释

**原则**: 公共 API 必须有文档，使用示例代码。

```rust
/// 数据库适配器 trait，定义了与数据库交互的核心接口。
///
/// # 示例
///
/// ```rust
/// use db_query_backend::{DatabaseAdapter, PostgresAdapter, ConnectionConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ConnectionConfig {
///         url: "postgresql://localhost/mydb".to_string(),
///         ..Default::default()
///     };
///
///     let adapter = PostgresAdapter::new(config).await?;
///     let result = adapter.execute_query("SELECT * FROM users LIMIT 10").await?;
///
///     println!("Found {} rows", result.rows.len());
///     Ok(())
/// }
/// ```
///
/// # 实现注意事项
///
/// - 所有实现必须是 Send + Sync
/// - 查询执行应该支持超时
/// - 错误应该提供足够的上下文信息
#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    /// 执行 SQL 查询并返回结果。
    ///
    /// # 参数
    ///
    /// - `sql`: 要执行的 SQL 查询字符串
    ///
    /// # 返回
    ///
    /// - `Ok(QueryResult)`: 查询成功，返回结果集
    /// - `Err(AppError)`: 查询失败，包含详细错误信息
    ///
    /// # 错误
    ///
    /// 可能返回以下错误类型：
    /// - `AppError::InvalidSql`: SQL 语法错误
    /// - `AppError::QueryTimeout`: 查询超时
    /// - `AppError::DatabaseError`: 数据库执行错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use db_query_backend::{DatabaseAdapter, PostgresAdapter};
    /// # async fn example(adapter: &PostgresAdapter) -> Result<(), Box<dyn std::error::Error>> {
    /// let result = adapter.execute_query("SELECT COUNT(*) FROM users").await?;
    /// println!("Query returned {} rows", result.rows.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, AppError>;

    /// 获取数据库元数据（表、列、类型等）。
    ///
    /// # 返回
    ///
    /// 包含数据库架构信息的 `Metadata` 对象。
    ///
    /// # 注意
    ///
    /// 元数据获取可能是昂贵的操作，建议使用缓存。
    async fn get_metadata(&self) -> Result<Metadata, AppError>;
}

/// SQL 验证器，确保查询安全性。
///
/// # 安全保证
///
/// - 只允许 SELECT 语句
/// - 自动添加 LIMIT 子句防止大结果集
/// - 拒绝潜在危险的 SQL 模式
///
/// # 示例
///
/// ```rust
/// use db_query_backend::SqlValidator;
///
/// let sql = "SELECT * FROM users";
/// let (validated, limit_added) = SqlValidator::validate_and_prepare(sql, 1000)?;
///
/// if limit_added {
///     println!("Automatically added LIMIT clause");
/// }
/// ```
pub struct SqlValidator;

/// 应用程序错误类型。
///
/// 使用 `thiserror` crate 自动实现 `Error` trait。
#[derive(Error, Debug)]
pub enum AppError {
    /// 数据库连接失败
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    /// 无效的 SQL 查询
    #[error("Invalid SQL query: {0}")]
    InvalidSql(String),
}
```

### 8.5 属性测试 (Property Testing)

**原则**: 使用 proptest 测试属性而非具体值。

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    // ✅ 好的实践：属性测试
    proptest! {
        #[test]
        fn test_limit_roundtrip(value in 1u32..=10000) {
            let limit = Limit::new(value).unwrap();
            assert_eq!(limit.value(), value);
        }

        #[test]
        fn test_sql_validator_preserves_select(
            table in "[a-z]{1,10}",
            columns in prop::collection::vec("[a-z]{1,10}", 1..5)
        ) {
            let sql = format!("SELECT {} FROM {}", columns.join(", "), table);
            let (validated, _) = SqlValidator::validate_and_prepare(&sql, 1000).unwrap();
            assert!(validated.contains(&table));
        }

        #[test]
        fn test_connection_id_roundtrip(uuid_str in "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}") {
            if let Ok(id) = ConnectionId::from_str(&uuid_str) {
                assert_eq!(id.to_string(), uuid_str);
            }
        }
    }
}
```

---

## 9. 项目结构

### 9.1 模块组织

**原则**: 清晰的模块层次，每个模块职责单一。

```
backend/src/
├── main.rs                      # 应用入口
├── lib.rs                       # 库入口，导出公共 API
├── api/                         # HTTP API 层
│   ├── mod.rs                   # 路由定义
│   ├── handlers/                # 请求处理器
│   │   ├── mod.rs
│   │   ├── connection.rs
│   │   ├── metadata.rs
│   │   └── query.rs
│   └── middleware.rs            # 中间件
├── models/                      # 数据模型
│   ├── mod.rs
│   ├── connection.rs
│   ├── metadata.rs
│   └── query.rs
├── services/                    # 业务逻辑层
│   ├── mod.rs
│   ├── db_service.rs
│   ├── query_service.rs
│   ├── llm_service.rs
│   ├── metadata_cache.rs
│   └── database/                # 数据库适配器
│       ├── mod.rs
│       ├── adapter.rs           # trait 定义
│       ├── postgresql.rs
│       ├── mysql.rs
│       ├── doris.rs
│       └── druid.rs
├── storage/                     # 持久化层
│   ├── mod.rs
│   └── sqlite.rs
├── validation/                  # 验证逻辑
│   ├── mod.rs
│   └── sql_validator.rs
├── config.rs                    # 配置管理
├── error.rs                     # 错误类型定义
└── utils.rs                     # 工具函数
```

```rust
// ✅ 好的实践：lib.rs 导出公共 API
pub mod api;
pub mod models;
pub mod services;
pub mod storage;
pub mod validation;
pub mod config;
pub mod error;

// 重新导出常用类型
pub use config::Config;
pub use error::AppError;
pub use models::{Connection, Metadata, QueryResult};
pub use services::database::DatabaseAdapter;

// ✅ 模块内部组织 (services/mod.rs)
mod db_service;
mod query_service;
mod llm_service;
mod metadata_cache;

pub mod database; // 子模块公开

pub use db_service::DatabaseService;
pub use query_service::QueryService;
pub use llm_service::LlmService;
pub use metadata_cache::MetadataCache;
```

### 9.2 可见性与封装

**原则**: 最小化公共 API 表面，内部实现细节私有化。

```rust
// ✅ 好的实践：精细的可见性控制
pub struct DatabaseService {
    connections: Arc<RwLock<HashMap<Uuid, Connection>>>, // 私有字段
    storage: Arc<SqliteStorage>,
}

impl DatabaseService {
    // 公共构造函数
    pub fn new(storage: Arc<SqliteStorage>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }

    // 公共方法
    pub async fn create_connection(&self, config: ConnectionConfig) -> Result<Connection, AppError> {
        let conn = self.establish_connection(&config).await?;
        self.store_connection(&conn).await?;
        Ok(conn)
    }

    // 私有辅助方法
    async fn establish_connection(&self, config: &ConnectionConfig) -> Result<Connection, AppError> {
        // 实现细节
        todo!()
    }

    // crate 内可见
    pub(crate) async fn store_connection(&self, conn: &Connection) -> Result<(), AppError> {
        self.storage.save(conn).await
    }
}

// ✅ 好的实践：模块内私有类型
mod internal {
    pub(super) struct InternalHelper {
        // 只在父模块可见
    }

    impl InternalHelper {
        pub(super) fn help(&self) {
            // ...
        }
    }
}
```

### 9.3 Feature Flags

**原则**: 使用 feature flags 控制可选功能。

```toml
# Cargo.toml

[features]
default = ["postgresql"]

# 数据库支持
postgresql = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
doris = ["dep:doris-client"]
druid = ["dep:druid-client"]

# LLM 集成
llm = ["dep:reqwest", "dep:serde_json"]

# 开发工具
dev-tools = ["tower-http/trace"]
```

```rust
// ✅ 好的实践：条件编译
#[cfg(feature = "postgresql")]
pub mod postgresql;

#[cfg(feature = "mysql")]
pub mod mysql;

// ✅ 运行时 feature 检测
pub fn supported_databases() -> Vec<&'static str> {
    let mut dbs = vec![];

    #[cfg(feature = "postgresql")]
    dbs.push("postgresql");

    #[cfg(feature = "mysql")]
    dbs.push("mysql");

    #[cfg(feature = "doris")]
    dbs.push("doris");

    dbs
}

// ✅ 好的实践：开发依赖
#[cfg(test)]
mod tests {
    #[cfg(feature = "dev-tools")]
    use pretty_assertions::assert_eq;
}
```

### 9.4 Workspace 管理

**原则**: 多 crate 项目使用 workspace 组织。

```toml
# 根目录 Cargo.toml
[workspace]
members = [
    "backend",
    "db-adapters",
    "sql-validator",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"

# backend/Cargo.toml
[package]
name = "db-query-backend"
version.workspace = true
edition.workspace = true

[dependencies]
tokio.workspace = true
serde.workspace = true
db-adapters = { path = "../db-adapters" }
sql-validator = { path = "../sql-validator" }
```

---

## 10. 代码质量

### 10.1 Clippy Lints

**原则**: 启用严格的 clippy 检查，修复所有警告。

```toml
# Cargo.toml 或 .cargo/config.toml

[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
# Pedantic 组
pedantic = "warn"
nursery = "warn"

# 性能
perf = "warn"

# 复杂度
cognitive_complexity = "warn"

# 特定 lints
unwrap_used = "deny"
expect_used = "warn"
panic = "deny"
todo = "warn"
```

```rust
// ✅ 好的实践：处理 clippy 警告
#[allow(clippy::module_name_repetitions)] // 有充分理由时允许
pub struct DatabaseDatabaseService {
    // ...
}

// ✅ 好的实践：避免 unwrap
pub fn get_config_value(key: &str) -> Result<String, ConfigError> {
    std::env::var(key)
        .map_err(|_| ConfigError::MissingKey(key.to_string()))
}

// ❌ 避免：使用 unwrap
pub fn bad_get_config(key: &str) -> String {
    std::env::var(key).unwrap() // clippy: unwrap_used
}
```

### 10.2 常见反模式

```rust
// ❌ 反模式 1：过度使用 clone
fn bad_clone(data: Vec<String>) -> Vec<String> {
    let mut result = data.clone();
    result.sort();
    result // 不需要 clone，直接消费 data
}

// ✅ 正确
fn good_sort(mut data: Vec<String>) -> Vec<String> {
    data.sort();
    data
}

// ❌ 反模式 2：字符串分配
fn bad_string() -> String {
    let s = "hello".to_string();
    s + " world" // 多次分配
}

// ✅ 正确
fn good_string() -> String {
    format!("{} {}", "hello", "world") // 一次分配
}

// ❌ 反模式 3：忽略 Result
async fn bad_async() {
    let _ = risky_operation().await; // 忽略错误
}

// ✅ 正确
async fn good_async() -> Result<(), AppError> {
    risky_operation().await?;
    Ok(())
}

// ❌ 反模式 4：类型转换链
fn bad_conversion(s: String) -> Vec<u8> {
    s.as_bytes().to_vec() // 不必要的拷贝
}

// ✅ 正确
fn good_conversion(s: String) -> Vec<u8> {
    s.into_bytes() // 零拷贝
}

// ❌ 反模式 5：布尔参数
fn bad_api(sql: &str, use_cache: bool, validate: bool, timeout: bool) {
    // 调用方需要记住参数顺序
}

// ✅ 正确：使用 builder 或配置结构
struct QueryOptions {
    use_cache: bool,
    validate: bool,
    timeout: Option<Duration>,
}

fn good_api(sql: &str, options: QueryOptions) {
    // 调用方使用命名字段
}
```

### 10.3 unsafe 使用原则

**原则**: 避免 unsafe，必要时充分文档化和审查。

```rust
// ✅ 好的实践：充分文档化 unsafe
/// # Safety
///
/// 调用者必须确保：
/// - `ptr` 是有效的非空指针
/// - `ptr` 指向的内存对齐且已初始化
/// - `ptr` 在函数执行期间不会被其他线程访问
/// - `len` 正确表示 `ptr` 指向的缓冲区长度
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> &'static [u8] {
    std::slice::from_raw_parts(ptr, len)
}

// ✅ 好的实践：最小化 unsafe 范围
pub fn safe_wrapper(data: Vec<u8>) -> String {
    // 安全的验证
    if !data.iter().all(|&b| b.is_ascii()) {
        return String::from_utf8_lossy(&data).to_string();
    }

    // 最小化 unsafe 块
    unsafe {
        // SAFETY: 我们已经验证了所有字节都是有效的 ASCII
        String::from_utf8_unchecked(data)
    }
}

// ❌ 避免：不必要的 unsafe
fn bad_unsafe() -> Vec<i32> {
    let mut v = Vec::new();
    unsafe {
        v.set_len(10); // 未初始化的内存！
    }
    v
}
```

### 10.4 依赖管理

**原则**: 谨慎选择依赖，定期更新，审查安全公告。

```toml
# ✅ 好的实践：指定版本范围
[dependencies]
tokio = "1.35"        # 允许补丁和次版本更新
serde = "1.0.195"     # 锁定次版本
sqlx = "=0.7.3"       # 精确版本（谨慎使用）

# ✅ 可选依赖
reqwest = { version = "0.11", optional = true }

# ✅ 开发依赖
[dev-dependencies]
criterion = "0.5"
proptest = "1.4"
mockall = "0.12"

# ✅ 使用 cargo-deny 检查依赖
# .cargo/deny.toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
```

```bash
# ✅ 定期更新依赖
cargo update

# ✅ 审计安全漏洞
cargo audit

# ✅ 检查未使用的依赖
cargo machete

# ✅ 生成依赖树
cargo tree
```

### 10.5 CI/CD 检查清单

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      # ✅ 格式检查
      - name: Check formatting
        run: cargo fmt -- --check

      # ✅ Clippy 检查
      - name: Clippy
        run: cargo clippy -- -D warnings

      # ✅ 运行测试
      - name: Run tests
        run: cargo test --all-features

      # ✅ 安全审计
      - name: Security audit
        run: cargo audit

      # ✅ 构建检查
      - name: Build release
        run: cargo build --release

      # ✅ 文档检查
      - name: Doc check
        run: cargo doc --no-deps --document-private-items
```

---

## 总结

本文档涵盖了 Rust 系统编程的核心设计原则，结合数据库查询工具项目的实际案例，提供了可操作的最佳实践指导。关键要点：

1. **所有权与借用**: 理解并正确使用 Rust 的核心特性
2. **异步编程**: 掌握 Tokio 运行时和 async/await 模式
3. **错误处理**: 使用 Result 和自定义错误类型
4. **类型系统**: 利用 trait、泛型和 newtype 提高安全性
5. **性能优化**: 避免不必要的克隆，使用零拷贝技术
6. **Web 服务**: Axum 框架的请求处理和状态管理
7. **数据库集成**: DataFusion 和 sqlx 的正确使用
8. **测试**: 单元测试、集成测试和属性测试
9. **项目组织**: 清晰的模块结构和可见性控制
10. **代码质量**: Clippy lints、避免反模式、依赖管理

遵循这些原则将帮助你编写安全、高效、可维护的 Rust 代码。

## 参考资源

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Tokio 文档](https://tokio.rs/)
- [Axum 指南](https://docs.rs/axum/)
- [sqlx 文档](https://docs.rs/sqlx/)
- [DataFusion 文档](https://arrow.apache.org/datafusion/)
- [Rust API 设计指南](https://rust-lang.github.io/api-guidelines/)
- [本项目 CLAUDE.md](./CLAUDE.md)
