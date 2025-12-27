// Data Export Utilities

/**
 * Convert query results to CSV format
 */
export function exportToCSV(data: any[], filename: string = 'query_results.csv') {
  if (!data || data.length === 0) {
    throw new Error('No data to export');
  }

  // Extract column headers from first row
  const headers = Object.keys(data[0]);

  // Create CSV content
  const csvRows: string[] = [];

  // Add header row
  csvRows.push(headers.map(escapeCSVValue).join(','));

  // Add data rows
  for (const row of data) {
    const values = headers.map(header => {
      const value = row[header];
      return escapeCSVValue(value);
    });
    csvRows.push(values.join(','));
  }

  const csvContent = csvRows.join('\n');

  // Add BOM for proper UTF-8 encoding in Excel
  const BOM = '\uFEFF';
  const blob = new Blob([BOM + csvContent], { type: 'text/csv;charset=utf-8;' });

  downloadBlob(blob, filename);
}

/**
 * Convert query results to JSON format
 */
export function exportToJSON(data: any[], filename: string = 'query_results.json') {
  if (!data || data.length === 0) {
    throw new Error('No data to export');
  }

  const jsonContent = JSON.stringify(data, null, 2);
  const blob = new Blob([jsonContent], { type: 'application/json;charset=utf-8;' });

  downloadBlob(blob, filename);
}

/**
 * Escape special characters for CSV format
 */
function escapeCSVValue(value: any): string {
  if (value === null || value === undefined) {
    return '';
  }

  // Convert to string
  let stringValue = String(value);

  // If the value contains comma, quote, or newline, wrap in quotes and escape quotes
  if (stringValue.includes(',') || stringValue.includes('"') || stringValue.includes('\n')) {
    stringValue = '"' + stringValue.replace(/"/g, '""') + '"';
  }

  return stringValue;
}

/**
 * Download blob as file
 */
function downloadBlob(blob: Blob, filename: string) {
  // Create download link
  const link = document.createElement('a');
  const url = URL.createObjectURL(blob);

  link.setAttribute('href', url);
  link.setAttribute('download', filename);
  link.style.visibility = 'hidden';

  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  // Clean up
  URL.revokeObjectURL(url);
}

/**
 * Generate filename with timestamp
 */
export function generateFilename(prefix: string = 'query_results', extension: string = 'csv'): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
  return `${prefix}_${timestamp}.${extension}`;
}
