use super::*;

pub struct CmdlineEditField {
    pub edit_field: EditField,
    pub active: bool,
}

impl CmdlineEditField {
    pub fn new() -> Self {
        Self {
            edit_field: EditField::new(),
            active: false,
        }
    }

    pub fn activate(&mut self, output_buffer: &OutputBuffer, cursor: &mut Cursor) {
        self.active = true;

        if let Some(cmdline) = output_buffer.user_provided_cmdline.as_ref() {
            self.edit_field.buffer = cmdline.clone();
        } else {
            self.edit_field.buffer.clear();
        }

        self.edit_field.cursor_x = self.edit_field.buffer.len();
        cursor.visible = true;
    }

    pub fn render(&self, row: Row, cursor: &mut Cursor) {
        if self.active {
            self.edit_field.render(row, cursor);
        }
    }

    pub fn accept_editing(&mut self, output_buffer: &mut OutputBuffer, cursor: &mut Cursor) {
        self.active = false;
        cursor.visible = false;
        output_buffer.user_provided_cmdline = Some(self.edit_field.buffer.clone());
        output_buffer.run_user_provided_cmdline();
    }

    pub fn cancel_editing(&mut self, cursor: &mut Cursor) {
        self.active = false;
        cursor.visible = false;
    }

    pub fn handle_key(
        &mut self,
        key: KeyStroke,
        key_map: &KeyMap,
        output_buffer: &mut OutputBuffer,
        cursor: &mut Cursor,
    ) {
        if self.active {
            match key {
                KeyStroke {
                    key: KEY_RETURN, ..
                } => {
                    self.accept_editing(output_buffer, cursor);
                }
                KeyStroke {
                    key: KEY_ESCAPE, ..
                } => {
                    self.cancel_editing(cursor);
                }
                _ => self.edit_field.handle_key(&key, key_map),
            }
        }
    }
}
