use std::cmp;
use std::iter::{once, repeat};

use crate::core::{Buffer, Location};
use crate::view::{rendering::RenderedBuffer, ViewGeometry};

use anyhow::Result;

pub struct Editor {
    buffer: Buffer,
    rendered_buffer: RenderedBuffer,
    cursor: Location,
    view: ViewGeometry,
}

impl Editor {
    pub fn new(width: usize, height: usize) -> Self {
        let buffer = Buffer::new();
        let rendered_buffer = RenderedBuffer::from(&buffer);

        Self {
            buffer,
            rendered_buffer,
            cursor: Location::new(0, 0),
            view: ViewGeometry::new(0, 0, width, height),
        }
    }

    pub fn get_view_cursor(&self) -> Location {
        Location::new(
            self.cursor.line - self.view.line,
            self.cursor.col - self.view.col,
        )
    }

    pub fn get_buffer_cursor(&self) -> Location {
        self.cursor
    }

    pub fn get_buffer_line_count(&self) -> usize {
        self.rendered_buffer.line_count()
    }

    pub fn get_view_width(&self) -> usize {
        self.view.width
    }

    pub fn get_view_height(&self) -> usize {
        self.view.height
    }

    pub fn get_view_contents(&self) -> impl Iterator<Item = String> {
        let ViewGeometry {
            line,
            col,
            width,
            height,
        } = self.view;
        let filler = once("~").chain(repeat(" ")).take(width).collect();
        self.rendered_buffer
            .get_view(line, col, width, height)
            .into_iter()
            .chain(repeat(filler))
            .take(height)
    }

    pub fn get_file_name(&self) -> Option<&String> {
        self.buffer.file_path()
    }

    pub fn open_file(&mut self, file_path: &str) -> Result<()> {
        self.buffer = Buffer::from_file(file_path)?;
        self.rendered_buffer = RenderedBuffer::from(&self.buffer);
        self.cursor = Location::new(0, 0);

        Ok(())
    }

    pub fn save_file(&mut self) -> Result<()> {
        self.buffer.save()
    }

    pub fn save_file_as(&mut self, file_path: &str) -> Result<()> {
        self.buffer.save_as(file_path)
    }

    pub fn remove_char_in_front(&mut self) {
        if self.is_cursor_at_eol_col() {
            if !self.is_cursor_at_buffer_bottom() {
                self.buffer.join_two_lines(self.cursor.line);

                self.rendered_buffer.remove_line(self.cursor.line + 1);
                self.rendered_buffer
                    .update_line(self.cursor.line, &self.buffer);
            }
        } else {
            self.buffer.remove_char(self.cursor);

            self.rendered_buffer
                .update_line(self.cursor.line, &self.buffer)
        }
    }

    pub fn remove_char_behind(&mut self) {
        if self.is_cursor_at_line_start() {
            if !self.is_cursor_at_buffer_top() {
                if self.is_cursor_at_view_top() {
                    self.move_view_up_unchecked()
                }

                self.cursor.line -= 1;
                self.cursor.col = self.rendered_buffer.eol_col(self.cursor.line);

                self.buffer.join_two_lines(self.cursor.line);

                self.rendered_buffer.remove_line(self.cursor.line + 1);
                self.rendered_buffer
                    .update_line(self.cursor.line, &self.buffer);
            }
        } else {
            self.move_cursor_left_unchecked();

            self.buffer.remove_char(self.cursor);
            self.rendered_buffer
                .update_line(self.cursor.line, &self.buffer)
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.buffer.insert_char(self.cursor, c);
        self.move_cursor_right_unchecked();

        self.rendered_buffer
            .update_line(self.cursor.line, &self.buffer);
    }

    fn move_cursor_left_unchecked(&mut self) {
        self.cursor.col -= 1
    }

    fn move_cursor_right_unchecked(&mut self) {
        self.cursor.col += 1
    }

    fn move_cursor_down_unchecked(&mut self) {
        self.cursor.line += 1
    }

    pub fn move_cursor_up(&mut self) {
        if self.is_cursor_at_buffer_top() {
            return;
        }

        if self.is_cursor_at_view_top() {
            self.move_view_up_unchecked();
        }

        self.cursor.line -= 1;
        self.adjust_cursor_past_eol();
    }

    pub fn insert_line(&mut self) {
        if self.is_cursor_at_line_start() {
            self.buffer.insert_line(self.cursor.line);
            self.rendered_buffer
                .insert_line(self.cursor.line, &self.buffer);
            self.move_cursor_down_unchecked();
        } else if self.is_cursor_at_eol_col() {
            let insert_index = self.cursor.line + 1;
            self.buffer.insert_line(insert_index);
            self.rendered_buffer.insert_line(insert_index, &self.buffer);
            self.move_cursor_down_unchecked();
        } else {
            self.buffer.split_line(self.cursor);
            self.rendered_buffer
                .update_line(self.cursor.line, &self.buffer);
            self.rendered_buffer
                .insert_line(self.cursor.line + 1, &self.buffer);
            self.move_cursor_down_unchecked();
            self.move_cursor_to_line_start();
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.is_cursor_at_buffer_bottom() {
            return;
        }

        if self.is_cursor_at_view_bottom() {
            self.move_view_down_unchecked();
        }

        self.cursor.line += 1;
        self.adjust_cursor_past_eol();
    }

    pub fn move_cursor_to_buffer_top(&mut self) {
        self.move_view_to_buffer_top();
        self.cursor.line = 0;
    }

    pub fn move_cursor_to_buffer_bottom(&mut self) {
        self.move_view_to_buffer_bottom();
        self.cursor.line = self.rendered_buffer.last_line();
    }

    fn move_view_to_buffer_top(&mut self) {
        self.view.line = 0;
    }

    fn move_view_to_buffer_bottom(&mut self) {
        self.view.line = self.bottom_most_view_pos()
    }

    fn bottom_most_view_pos(&self) -> usize {
        // TODO! See if the interaction between the rendered buffer and the view
        // can be expressed in a better way.
        self.rendered_buffer
            .line_count()
            .saturating_sub(self.view.height)
    }

    pub fn move_one_view_up(&mut self) {
        let cursor_line_offset = self.cursor.line - self.view.line;
        self.view.line = self.view.line.saturating_sub(self.view.height);
        self.cursor.line = self.view.line + cursor_line_offset;
        self.adjust_cursor_past_eol();
    }

    pub fn move_one_view_down(&mut self) {
        let cursor_line_offset = self.cursor.line - self.view.line;
        self.view.line = cmp::min(
            self.view.line + self.view.height,
            self.bottom_most_view_pos(),
        );
        self.cursor.line = self.view.line + cursor_line_offset;
        self.adjust_cursor_past_eol();
    }

    pub fn move_cursor_left(&mut self) {
        if self.is_cursor_at_line_start() {
            if self.is_cursor_at_buffer_top() {
                return;
            }

            self.move_cursor_up();
            self.move_cursor_to_eol_col();
        } else {
            if self.is_cursor_at_view_left() {
                self.move_view_left_unchecked();
            }

            self.move_cursor_left_unchecked();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.is_cursor_at_eol_col() {
            if self.is_cursor_at_buffer_bottom() {
                return;
            }

            self.move_cursor_down();
            self.move_cursor_to_line_start();
        } else {
            if self.is_cursor_at_view_right() {
                self.move_view_right_unchecked();
            }

            self.move_cursor_right_unchecked();
        }
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.cursor.col = 0;
    }

    pub fn move_cursor_to_line_end(&mut self) {
        self.cursor.col = self.rendered_buffer.last_col(self.cursor.line);
    }

    fn move_cursor_to_eol_col(&mut self) {
        self.cursor.col = self.rendered_buffer.eol_col(self.cursor.line);
    }

    fn adjust_cursor_past_eol(&mut self) {
        if self.is_cursor_past_eol() {
            self.move_cursor_to_eol_col();
        }
    }

    fn move_view_up_unchecked(&mut self) {
        self.view.line -= 1;
    }

    fn move_view_down_unchecked(&mut self) {
        self.view.line += 1;
    }

    fn move_view_left_unchecked(&mut self) {
        self.view.col -= 1;
    }

    fn move_view_right_unchecked(&mut self) {
        self.view.col += 1;
    }

    fn is_cursor_at_view_top(&self) -> bool {
        self.cursor.line == self.view.line
    }

    fn is_cursor_at_view_bottom(&self) -> bool {
        self.cursor.line == self.view.last_line()
    }

    fn is_cursor_at_view_left(&self) -> bool {
        self.cursor.col == self.view.col
    }

    fn is_cursor_at_view_right(&self) -> bool {
        self.cursor.col == self.view.last_col()
    }

    fn is_cursor_at_buffer_top(&self) -> bool {
        self.cursor.line == 0
    }

    fn is_cursor_at_buffer_bottom(&self) -> bool {
        self.cursor.line == self.rendered_buffer.last_line()
    }

    fn is_cursor_at_line_start(&self) -> bool {
        self.cursor.col == 0
    }

    fn is_cursor_at_eol_col(&self) -> bool {
        self.cursor.col == self.rendered_buffer.eol_col(self.cursor.line)
    }

    fn is_cursor_past_eol(&self) -> bool {
        self.cursor.col > self.rendered_buffer.eol_col(self.cursor.line)
    }
}
