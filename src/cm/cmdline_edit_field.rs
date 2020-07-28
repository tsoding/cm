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

    pub fn activate(&mut self, line_list: &LineList, global: &mut Global) {
        self.active = true;

        if let Some(cmdline) = line_list.user_provided_cmdline.as_ref() {
            self.edit_field.buffer = cmdline.clone();
        } else {
            self.edit_field.buffer.clear();
        }

        self.edit_field.cursor_x = self.edit_field.buffer.len();
        global.cursor_visible = true;
    }

    pub fn render(&self, row: Row, global: &mut Global) {
        if self.active {
            self.edit_field.render(row);
            global.cursor_x = self.edit_field.cursor_x as i32;
            global.cursor_y = row.y as i32;
        }
    }

    pub fn accept_editing(&mut self, line_list: &mut LineList, global: &mut Global) {
        self.active = false;
        global.cursor_visible = false;
        line_list.user_provided_cmdline = Some(self.edit_field.buffer.clone());
        line_list.run_user_provided_cmdline();
    }

    pub fn cancel_editing(&mut self, global: &mut Global) {
        self.active = false;
        global.cursor_visible = false;
    }

    pub fn handle_key(&mut self, key: KeyStroke, line_list: &mut LineList, global: &mut Global) {
        if self.active {
            match key {
                KeyStroke {
                    key: KEY_RETURN, ..
                } => {
                    self.accept_editing(line_list, global);
                }
                KeyStroke {
                    key: KEY_ESCAPE, ..
                } => {
                    self.cancel_editing(global);
                }
                _ => self.edit_field.handle_key(key),
            }
        }
    }
}
