# Instructions

## constitution


## 基本思路
这是一个数据库查询工具，用户可以添加一个db url, 系统会连接到数据库，获取数据库的metadata, 然后将数据库中的table和view的信息展示出来，然后用户可以自己输入sql查询，也可以通过自然语言来生成sql查询。

基本想法：
- 数据库链接串和数据库的metadata都会存储到sqlite数据库中。我们可以根据postgres的功能来查询系统中的表和视图的信息，然后用LLM来将这些信息转换成json格式，然后存储到sqlite数据库中，这个信息以后可以复用。
- 当用户使用LLM来生成sql查询时，我们可以把系统中的表和视图的信息作为context传递给LLM， 然后LLM会根据这些信息来生成sql查询。
- 任何输入的sql语句，都需要经过sqlparser解析，确保语法正确， 并且仅包含select语句。 如果语法不正确，需要给出错误信息。
    - 如果查询不包含limit子句，则默认添加limit 1000子句
- 输出格式是json，前端将其组织成表格，并显示出来。 

后端是使用rust / axum/tokio / datafusion/ rig
前端是使用 react/ refine 5/tailwind/ ant design来实现。sql editor使用monaco editor来实现。