-- Create todo-list database
CREATE DATABASE IF NOT EXISTS todolist CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

USE todolist;

-- Users table
CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    full_name VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE,
    INDEX idx_username (username),
    INDEX idx_email (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Categories table
CREATE TABLE categories (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    color VARCHAR(7) DEFAULT '#3B82F6',
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_name (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Todos table
CREATE TABLE todos (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    category_id INT,
    title VARCHAR(200) NOT NULL,
    description TEXT,
    status ENUM('pending', 'in_progress', 'completed', 'cancelled') DEFAULT 'pending',
    priority ENUM('low', 'medium', 'high', 'urgent') DEFAULT 'medium',
    due_date DATE,
    completed_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL,
    INDEX idx_user_id (user_id),
    INDEX idx_category_id (category_id),
    INDEX idx_status (status),
    INDEX idx_priority (priority),
    INDEX idx_due_date (due_date)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Tags table
CREATE TABLE tags (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(30) NOT NULL UNIQUE,
    color VARCHAR(7) DEFAULT '#6B7280',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_name (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Todo tags junction table
CREATE TABLE todo_tags (
    todo_id INT NOT NULL,
    tag_id INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (todo_id, tag_id),
    FOREIGN KEY (todo_id) REFERENCES todos(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Comments table
CREATE TABLE comments (
    id INT AUTO_INCREMENT PRIMARY KEY,
    todo_id INT NOT NULL,
    user_id INT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (todo_id) REFERENCES todos(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_todo_id (todo_id),
    INDEX idx_user_id (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Insert sample users
INSERT INTO users (username, email, full_name, is_active) VALUES
('alice', 'alice@example.com', 'Alice Johnson', TRUE),
('bob', 'bob@example.com', 'Bob Smith', TRUE),
('charlie', 'charlie@example.com', 'Charlie Brown', TRUE),
('diana', 'diana@example.com', 'Diana Prince', FALSE);

-- Insert sample categories
INSERT INTO categories (name, color, description) VALUES
('Work', '#3B82F6', 'Work related tasks'),
('Personal', '#10B981', 'Personal tasks and errands'),
('Shopping', '#F59E0B', 'Shopping list items'),
('Health', '#EF4444', 'Health and fitness goals'),
('Learning', '#8B5CF6', 'Educational and learning tasks');

-- Insert sample tags
INSERT INTO tags (name, color) VALUES
('urgent', '#EF4444'),
('important', '#F59E0B'),
('quick', '#10B981'),
('meeting', '#3B82F6'),
('home', '#8B5CF6'),
('office', '#6366F1'),
('reminder', '#EC4899'),
('bug', '#DC2626'),
('feature', '#059669'),
('documentation', '#0891B2');

-- Insert sample todos
INSERT INTO todos (user_id, category_id, title, description, status, priority, due_date) VALUES
-- Alice's todos
(1, 1, 'Complete project proposal', 'Finish the Q1 project proposal and submit to management', 'in_progress', 'high', DATE_ADD(CURDATE(), INTERVAL 3 DAY)),
(1, 1, 'Review pull requests', 'Review and merge pending pull requests in GitHub', 'pending', 'medium', DATE_ADD(CURDATE(), INTERVAL 1 DAY)),
(1, 2, 'Buy groceries', 'Milk, eggs, bread, vegetables', 'pending', 'low', CURDATE()),
(1, 4, 'Gym workout', 'Complete leg day workout', 'completed', 'medium', DATE_SUB(CURDATE(), INTERVAL 1 DAY)),
(1, 5, 'Read React documentation', 'Study React 18 new features', 'in_progress', 'medium', DATE_ADD(CURDATE(), INTERVAL 7 DAY)),

-- Bob's todos
(2, 1, 'Client meeting preparation', 'Prepare slides and demo for client presentation', 'in_progress', 'urgent', DATE_ADD(CURDATE(), INTERVAL 2 DAY)),
(2, 1, 'Update project timeline', 'Revise project timeline based on recent changes', 'pending', 'high', DATE_ADD(CURDATE(), INTERVAL 5 DAY)),
(2, 3, 'Order office supplies', 'Pens, notebooks, printer paper', 'pending', 'low', DATE_ADD(CURDATE(), INTERVAL 10 DAY)),
(2, 2, 'Fix kitchen sink', 'Call plumber or attempt DIY repair', 'pending', 'medium', DATE_ADD(CURDATE(), INTERVAL 3 DAY)),

-- Charlie's todos
(3, 5, 'Learn MySQL basics', 'Complete MySQL tutorial on database design', 'in_progress', 'medium', DATE_ADD(CURDATE(), INTERVAL 14 DAY)),
(3, 1, 'Write unit tests', 'Add test coverage for new authentication module', 'pending', 'high', DATE_ADD(CURDATE(), INTERVAL 4 DAY)),
(3, 2, 'Birthday party planning', 'Plan surprise party for Diana', 'pending', 'high', DATE_ADD(CURDATE(), INTERVAL 8 DAY)),
(3, 4, 'Schedule dental checkup', 'Book appointment with Dr. Smith', 'pending', 'low', DATE_ADD(CURDATE(), INTERVAL 20 DAY)),

-- Diana's todos (inactive user)
(4, 2, 'Clean garage', 'Organize and clean garage over weekend', 'cancelled', 'low', DATE_SUB(CURDATE(), INTERVAL 5 DAY)),
(4, 1, 'Update resume', 'Update resume with recent projects', 'completed', 'medium', DATE_SUB(CURDATE(), INTERVAL 10 DAY));

-- Update completed todos with completion dates
UPDATE todos SET completed_at = DATE_SUB(NOW(), INTERVAL 1 DAY) WHERE status = 'completed' AND user_id = 1;
UPDATE todos SET completed_at = DATE_SUB(NOW(), INTERVAL 10 DAY) WHERE status = 'completed' AND user_id = 4;

-- Insert todo-tag relationships
INSERT INTO todo_tags (todo_id, tag_id) VALUES
(1, 2),  -- Complete project proposal -> important
(1, 9),  -- Complete project proposal -> feature
(2, 3),  -- Review pull requests -> quick
(3, 1),  -- Buy groceries -> urgent
(3, 5),  -- Buy groceries -> home
(6, 1),  -- Client meeting preparation -> urgent
(6, 4),  -- Client meeting preparation -> meeting
(7, 2),  -- Update project timeline -> important
(10, 5), -- Learn MySQL basics -> home
(11, 8), -- Write unit tests -> bug
(12, 7); -- Birthday party planning -> reminder

-- Insert sample comments
INSERT INTO comments (todo_id, user_id, content) VALUES
(1, 1, 'Started working on the executive summary section'),
(1, 2, 'Make sure to include the budget breakdown'),
(1, 1, 'Will add budget details by tomorrow'),
(6, 2, 'Demo environment is ready for presentation'),
(6, 1, 'Great! Let me know if you need help with slides'),
(10, 3, 'Completed chapters 1-3, moving to JOIN operations next'),
(11, 2, 'Don\'t forget to test edge cases'),
(12, 3, 'Venue booked for next Saturday!');

-- Create a view for active todos summary
CREATE VIEW active_todos_summary AS
SELECT
    u.username,
    u.email,
    c.name AS category,
    t.title,
    t.status,
    t.priority,
    t.due_date,
    DATEDIFF(t.due_date, CURDATE()) AS days_until_due,
    GROUP_CONCAT(tg.name ORDER BY tg.name SEPARATOR ', ') AS tags
FROM todos t
JOIN users u ON t.user_id = u.id
LEFT JOIN categories c ON t.category_id = c.id
LEFT JOIN todo_tags tt ON t.id = tt.todo_id
LEFT JOIN tags tg ON tt.tag_id = tg.id
WHERE t.status IN ('pending', 'in_progress')
    AND u.is_active = TRUE
GROUP BY t.id, u.username, u.email, c.name, t.title, t.status, t.priority, t.due_date
ORDER BY
    CASE t.priority
        WHEN 'urgent' THEN 1
        WHEN 'high' THEN 2
        WHEN 'medium' THEN 3
        WHEN 'low' THEN 4
    END,
    t.due_date;

-- Create a view for user statistics
CREATE VIEW user_stats AS
SELECT
    u.id,
    u.username,
    u.email,
    COUNT(t.id) AS total_todos,
    SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END) AS completed_todos,
    SUM(CASE WHEN t.status = 'pending' THEN 1 ELSE 0 END) AS pending_todos,
    SUM(CASE WHEN t.status = 'in_progress' THEN 1 ELSE 0 END) AS in_progress_todos,
    SUM(CASE WHEN t.status = 'cancelled' THEN 1 ELSE 0 END) AS cancelled_todos,
    MAX(t.created_at) AS last_todo_created
FROM users u
LEFT JOIN todos t ON u.id = t.user_id
WHERE u.is_active = TRUE
GROUP BY u.id, u.username, u.email;

-- Show summary
SELECT 'Database initialization complete!' AS status;
SELECT COUNT(*) AS total_users FROM users;
SELECT COUNT(*) AS total_categories FROM categories;
SELECT COUNT(*) AS total_todos FROM todos;
SELECT COUNT(*) AS total_tags FROM tags;
SELECT COUNT(*) AS total_comments FROM comments;
