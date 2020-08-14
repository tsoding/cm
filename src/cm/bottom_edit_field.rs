use super::*;

pub struct BottomEditField {
    pub edit_field: EditField,
}

impl BottomEditField {
    pub fn new() -> Self {
        Self {
            edit_field: EditField::new(),
        }
    }

    pub fn activate(&mut self, cursor: &mut Cursor, value: String) {
        self.edit_field.buffer = value;
        self.edit_field.cursor_x = self.edit_field.buffer.len();
        cursor.visible = true;
    }

    pub fn render(&self, row: Row, cursor: &mut Cursor) {
        self.edit_field.render(row, cursor);
    }

    pub fn stop_editing(&mut self, cursor: &mut Cursor) {
        cursor.visible = false;
    }

    pub fn handle_key(&mut self, key: KeyStroke, key_map: &KeyMap) {
        self.edit_field.handle_key(key, key_map);
    }
}
