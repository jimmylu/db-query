// SQL Formatter Utility
// Simple SQL formatter without external dependencies

/**
 * Format SQL query with proper indentation and line breaks
 */
export function formatSQL(sql: string): string {
  if (!sql || !sql.trim()) {
    return sql;
  }

  // Remove extra whitespace
  let formatted = sql.trim().replace(/\s+/g, ' ');

  // Keywords that should start on a new line
  const newLineKeywords = [
    'SELECT',
    'FROM',
    'WHERE',
    'GROUP BY',
    'HAVING',
    'ORDER BY',
    'LIMIT',
    'OFFSET',
    'UNION',
    'UNION ALL',
    'INNER JOIN',
    'LEFT JOIN',
    'RIGHT JOIN',
    'FULL JOIN',
    'CROSS JOIN',
    'ON',
    'AND',
    'OR',
  ];

  // Add newlines before major keywords
  newLineKeywords.forEach((keyword) => {
    const regex = new RegExp(`\\b${keyword}\\b`, 'gi');
    formatted = formatted.replace(regex, (match) => `\n${match}`);
  });

  // Split into lines
  const lines = formatted.split('\n').filter(line => line.trim());

  // Format each line with proper indentation
  let indentLevel = 0;
  const formattedLines: string[] = [];
  const indent = '  '; // 2 spaces

  for (let i = 0; i < lines.length; i++) {
    let line = lines[i].trim();

    // Decrease indent for closing parentheses
    if (line.startsWith(')')) {
      indentLevel = Math.max(0, indentLevel - 1);
    }

    // Add the line with current indentation
    if (line) {
      // Special handling for certain keywords
      if (line.match(/^(AND|OR)\b/i) && indentLevel > 0) {
        formattedLines.push(indent.repeat(indentLevel) + line);
      } else if (line.match(/^(INNER JOIN|LEFT JOIN|RIGHT JOIN|FULL JOIN|CROSS JOIN)\b/i)) {
        formattedLines.push(indent.repeat(Math.max(1, indentLevel)) + line);
      } else if (line.match(/^(ON)\b/i)) {
        formattedLines.push(indent.repeat(Math.max(1, indentLevel + 1)) + line);
      } else if (line.match(/^(WHERE|GROUP BY|HAVING|ORDER BY|LIMIT|OFFSET)\b/i)) {
        formattedLines.push(indent.repeat(Math.max(0, indentLevel)) + line);
      } else if (line.match(/^(SELECT|FROM|UNION|UNION ALL)\b/i)) {
        formattedLines.push(indent.repeat(indentLevel) + line);
      } else {
        formattedLines.push(indent.repeat(indentLevel) + line);
      }
    }

    // Increase indent for opening parentheses
    const openParens = (line.match(/\(/g) || []).length;
    const closeParens = (line.match(/\)/g) || []).length;
    indentLevel += openParens - closeParens;
    indentLevel = Math.max(0, indentLevel);
  }

  // Join lines and clean up
  let result = formattedLines.join('\n');

  // Format comma-separated lists in SELECT
  result = result.replace(/SELECT\s+(.+?)\s+FROM/gi, (match, columns) => {
    if (columns.includes(',')) {
      const columnList = columns.split(',').map((col: string) => col.trim());
      if (columnList.length > 3) {
        // More than 3 columns, put each on new line
        return `SELECT\n  ${columnList.join(',\n  ')}\nFROM`;
      }
    }
    return match;
  });

  // Clean up extra blank lines
  result = result.replace(/\n{3,}/g, '\n\n');

  return result;
}

/**
 * Uppercase all SQL keywords
 */
export function uppercaseKeywords(sql: string): string {
  const keywords = [
    'SELECT', 'FROM', 'WHERE', 'AND', 'OR', 'NOT', 'IN', 'BETWEEN', 'LIKE',
    'IS', 'NULL', 'AS', 'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER', 'FULL',
    'CROSS', 'ON', 'GROUP', 'BY', 'HAVING', 'ORDER', 'LIMIT', 'OFFSET',
    'UNION', 'ALL', 'DISTINCT', 'COUNT', 'SUM', 'AVG', 'MAX', 'MIN',
    'CASE', 'WHEN', 'THEN', 'ELSE', 'END', 'ASC', 'DESC',
  ];

  let result = sql;
  keywords.forEach((keyword) => {
    const regex = new RegExp(`\\b${keyword}\\b`, 'gi');
    result = result.replace(regex, keyword);
  });

  return result;
}

/**
 * Format and uppercase SQL
 */
export function formatAndUppercaseSQL(sql: string): string {
  const formatted = formatSQL(sql);
  return uppercaseKeywords(formatted);
}
