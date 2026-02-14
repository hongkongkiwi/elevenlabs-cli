//! Table formatting utilities using comfy_table

use comfy_table::{Table, presets, Attribute, Cell, Color};

/// Create a new table with standard CLI styling
pub fn create_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    
    // Add styled headers
    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| Cell::new(*h).add_attribute(Attribute::Bold))
        .collect();
    table.set_header(header_cells);
    
    table
}

/// Add a row to a table with optional coloring
pub fn add_row(table: &mut Table, cells: Vec<String>) {
    table.add_row(cells);
}

/// Add a row with the first cell highlighted as an ID
pub fn add_row_with_id(table: &mut Table, id: &str, rest: Vec<String>) {
    let mut row = vec![Cell::new(id).fg(Color::Yellow)];
    for cell in rest {
        row.push(Cell::new(cell));
    }
    table.add_row(row);
}

/// Add a row with name highlighted in cyan
pub fn add_row_with_name(table: &mut Table, name: &str, rest: Vec<String>) {
    let mut row = vec![Cell::new(name).fg(Color::Cyan)];
    for cell in rest {
        row.push(Cell::new(cell));
    }
    table.add_row(row);
}
