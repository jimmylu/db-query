# MySQL Todo-List 测试数据库

## 连接信息

**Docker 容器名称**: `mysql-todolist`
**连接 URL**: `mysql://root:password123@localhost:3306/todolist`

### 使用应用程序连接

在浏览器中打开 http://localhost:5173，填写以下信息：

- **连接名称**: MySQL Todo List
- **数据库类型**: MySQL
- **连接 URL**: `mysql://root:password123@localhost:3306/todolist`

## 数据库结构

### 表 (Tables)

1. **users** - 用户表
   - 4 个用户 (alice, bob, charlie, diana)
   - diana 是非活跃用户

2. **categories** - 分类表
   - Work (工作)
   - Personal (个人)
   - Shopping (购物)
   - Health (健康)
   - Learning (学习)

3. **todos** - 待办事项表
   - 15 个待办事项
   - 包含各种状态: pending, in_progress, completed, cancelled
   - 优先级: urgent, high, medium, low

4. **tags** - 标签表
   - 10 个标签 (urgent, important, quick, meeting, home, office, reminder, bug, feature, documentation)

5. **comments** - 评论表
   - 8 条评论

6. **todo_tags** - 待办事项-标签关联表

### 视图 (Views)

1. **active_todos_summary** - 活跃待办事项汇总
   - 显示所有活跃用户的待办事项
   - 包含用户信息、分类、标签等

2. **user_stats** - 用户统计
   - 每个用户的待办事项统计

## 示例查询

### 基础查询

```sql
-- 查看所有用户
SELECT * FROM users;

-- 查看所有待办事项
SELECT * FROM todos LIMIT 10;

-- 查看活跃待办事项
SELECT * FROM active_todos_summary;

-- 查看用户统计
SELECT * FROM user_stats;
```

### 复杂查询

```sql
-- 查找即将到期的高优先级任务
SELECT
    u.username,
    t.title,
    t.priority,
    t.due_date,
    DATEDIFF(t.due_date, CURDATE()) AS days_left
FROM todos t
JOIN users u ON t.user_id = u.id
WHERE t.status IN ('pending', 'in_progress')
    AND t.priority IN ('urgent', 'high')
    AND t.due_date <= DATE_ADD(CURDATE(), INTERVAL 7 DAY)
ORDER BY t.due_date;

-- 按分类统计待办事项
SELECT
    c.name AS category,
    COUNT(*) AS total,
    SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END) AS completed,
    SUM(CASE WHEN t.status = 'pending' THEN 1 ELSE 0 END) AS pending
FROM todos t
JOIN categories c ON t.category_id = c.id
GROUP BY c.name;

-- 查找有评论的待办事项
SELECT
    t.title,
    u.username,
    COUNT(c.id) AS comment_count,
    MAX(c.created_at) AS last_comment_at
FROM todos t
JOIN users u ON t.user_id = u.id
LEFT JOIN comments c ON t.id = c.todo_id
GROUP BY t.id, t.title, u.username
HAVING comment_count > 0;
```

### 使用标签查询

```sql
-- 查找带有 'urgent' 标签的待办事项
SELECT
    t.title,
    u.username,
    t.status,
    GROUP_CONCAT(tg.name) AS tags
FROM todos t
JOIN users u ON t.user_id = u.id
JOIN todo_tags tt ON t.id = tt.todo_id
JOIN tags tg ON tt.tag_id = tg.id
WHERE tg.name = 'urgent'
GROUP BY t.id, t.title, u.username, t.status;
```

## 自然语言查询示例

在应用程序中，您可以使用以下自然语言问题：

1. "显示所有用户"
2. "有多少个待办事项？"
3. "显示所有高优先级的待办事项"
4. "哪些任务是紧急的？"
5. "Alice 有多少个待办事项？"
6. "显示本周到期的任务"
7. "统计每个分类的待办事项数量"
8. "显示所有已完成的任务"
9. "哪些任务有评论？"
10. "显示工作相关的待办事项"

## Docker 管理命令

```bash
# 查看容器状态
docker ps -a | grep mysql-todolist

# 查看容器日志
docker logs mysql-todolist

# 进入 MySQL 命令行
docker exec -it mysql-todolist mysql -uroot -ppassword123 todolist

# 停止容器
docker stop mysql-todolist

# 启动容器
docker start mysql-todolist

# 删除容器
docker rm -f mysql-todolist

# 重新创建容器（如果需要重置数据）
docker rm -f mysql-todolist
docker run -d \
  --name mysql-todolist \
  -e MYSQL_ROOT_PASSWORD=password123 \
  -e MYSQL_DATABASE=todolist \
  -p 3306:3306 \
  -v /Users/jimmylu/Documents/example_project/db_query/fixtures/mysql-init.sql:/docker-entrypoint-initdb.d/init.sql:ro \
  mysql:latest
```

## 数据统计

- **用户数**: 4 (3 活跃)
- **分类数**: 5
- **待办事项数**: 15
- **标签数**: 10
- **评论数**: 8
- **视图数**: 2

## 测试建议

1. 连接数据库并查看元数据
2. 执行简单的 SELECT 查询
3. 测试使用自然语言生成 SQL
4. 验证 LIMIT 子句自动添加功能
5. 测试复杂的 JOIN 查询
6. 测试视图查询
7. 验证 MySQL 特定语法（如 CONCAT, CURDATE()）

## 注意事项

- 密码在命令行中可见，仅用于开发测试
- 数据库端口 3306 已映射到主机
- 初始化脚本在 `fixtures/mysql-init.sql`
- 使用 UTF8MB4 字符集，支持中文和 emoji
