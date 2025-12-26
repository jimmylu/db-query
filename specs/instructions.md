# Instructions

## 基本思路

这是一个数据库查询工具，用户可以添加一个db url, 系统会连接到数据库，获取数据库的metadata, 然后将数据库中的table和view的信息展示出来，然后用户可以自己输入sql查询，也可以通过自然语言来生成sql查询。

基本想法：

- 数据库链接串和数据库的metadata都会存储到sqlite数据库中。我们可以根据postgres的功能来查询系统中的表和视图的信息，然后用LLM来将这些信息转换成json格式，然后存储到sqlite数据库中，这个信息以后可以复用。
- 当用户使用LLM来生成sql查询时，我们可以把系统中的表和视图的信息作为context传递给LLM， 然后LLM会根据这些信息来生成sql查询。
- 任何输入的sql语句，都需要经过sqlparser解析，确保语法正确， 并且仅包含select语句。 如果语法不正确，需要给出错误信息。
  - 如果查询不包含limit子句，则默认添加limit 1000子句
- 输出格式是json，前端将其组织成表格，并显示出来。

后端是使用rust / axum/tokio / datafusion/ rig.rs(llm gateway)
前端是使用 react/ refine 5/tailwind/ ant design来实现。sql editor使用monaco editor来实现。


### 重构
以datafusion为中间语义层，支持扩展mysql，doris，apache druid等异构数据库

### code review command
帮我参照 @.cursor/commands/speckit.specify.md的结构，think ultra hard, 构建一个rust和vue代码进行深度代码审查的命令，放在@.claude/commands/下，主要考虑几个方面：
- 架构设计： 是否考虑rust架构设计的最佳实践？是否有清晰的接口设计？是否考虑一定程度的扩展性；
- KISS原则
- 代码质量:DRY,YAGNI, SOLID, etc.
- 使用builder模式


## 添加 MySQL db 支持
参考PostgresQL实现，实现 MysQL的 metadata提取和查询支持，同时自然语言生成sql 也支持 MySQL

## 安装测试mysql库，并初始化表和数据
用docker拉取一个最新版本的mysql镜像，并初始化一个样例todo-list库（包括表和数据）

## 统一sql语义重构
- 要求将datafusion作为统一的sql语义层，对用户使用同一套sql语法，对异构的引擎，会自动进行方言的翻译
- 要求具备可扩展性，能够容易支持更多的数据库
