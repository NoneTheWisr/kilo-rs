use std::cmp;
use std::iter::{once, repeat};

use crate::core::{Buffer, Location};
use crate::view::{rendering::RenderedBuffer, ViewGeometry};

use anyhow::Result;

pub struct Editor {
    buffer: Buffer,
    view: ViewGeometry,
    rendered_buffer: RenderedBuffer,
    cursor: Location,
}

impl Editor {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            buffer: Buffer::empty(),
            view: ViewGeometry::new(0, 0, width, height),
            rendered_buffer: RenderedBuffer::empty(),
            cursor: Location::new(0, 0),
        }
    }

    pub fn get_view_relative_cursor_position(&self) -> Location {
        Location::new(
            self.cursor.row - self.view.row,
            self.cursor.col - self.view.col,
        )
    }

    pub fn get_view_contents(&self) -> impl Iterator<Item = String> {
        let ViewGeometry {
            row,
            col,
            width,
            height,
        } = self.view;
        let filler = once("~").chain(repeat(" ")).take(width).collect();
        self.rendered_buffer
            .rect(row, col, width, height)
            .into_iter()
            .chain(repeat(filler))
            .take(height)
    }

    pub fn open_file(&mut self, file_path: &str) -> Result<()> {
        self.buffer = Buffer::from_file(file_path)?;
        self.rendered_buffer = RenderedBuffer::render(&self.buffer);
        self.cursor = Location::new(0, 0);

        Ok(())
    }

    pub fn move_cursor_up(&mut self) {
        if self.is_cursor_at_buffer_top() {
            return;
        }

        if self.is_cursor_at_view_top() {
            self.move_view_up_unchecked();
        }

        self.cursor.row -= 1;
        self.adjust_cursor_past_eol();
    }

    pub fn move_cursor_down(&mut self) {
        if self.is_cursor_at_buffer_bottom() {
            return;
        }

        if self.is_cursor_at_view_bottom() {
            self.move_view_down_unchecked();
        }

        self.cursor.row += 1;
        self.adjust_cursor_past_eol();
    }

    pub fn move_cursor_to_buffer_top(&mut self) {
        self.move_view_to_buffer_top();
        self.cursor.row = 0;
    }

    pub fn move_cursor_to_buffer_bottom(&mut self) {
        self.move_view_to_buffer_bottom();
        self.cursor.row = self.rendered_buffer.last_line();
    }

    fn move_view_to_buffer_top(&mut self) {
        self.view.row = 0;
    }

    fn move_view_to_buffer_bottom(&mut self) {
        self.view.row = self.bottom_most_view_pos()
    }

    fn bottom_most_view_pos(&self) -> usize {
        // TODO! See if the interaction between the rendered buffer and the view
        // can be expressed in a better way.
        self.rendered_buffer
            .line_count()
            .saturating_sub(self.view.height)
    }

    pub fn move_one_view_up(&mut self) {
        let cursor_row_offset = self.cursor.row - self.view.row;
        self.view.row = self.view.row.saturating_sub(self.view.height);
        self.cursor.row = self.view.row + cursor_row_offset;
        self.adjust_cursor_past_eol();
    }

    pub fn move_one_view_down(&mut self) {
        let cursor_row_offset = self.cursor.row - self.view.row;
        self.view.row = cmp::min(
            self.view.row + self.view.height,
            self.bottom_most_view_pos(),
        );
        self.cursor.row = self.view.row + cursor_row_offset;
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

            self.cursor.col -= 1;
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

            self.cursor.col += 1;
        }
    }

    pub fn move_cursor_to_line_start(&mut self) {
        self.cursor.col = 0;
    }

    pub fn move_cursor_to_line_end(&mut self) {
        self.cursor.col = self.rendered_buffer.last_col(self.cursor.row);
    }

    fn move_cursor_to_eol_col(&mut self) {
        self.cursor.col = self.rendered_buffer.eol_col(self.cursor.row);
    }

    fn adjust_cursor_past_eol(&mut self) {
        if self.is_cursor_past_eol() {
            self.move_cursor_to_eol_col();
        }
    }

    fn move_view_up_unchecked(&mut self) {
        self.view.row -= 1;
    }

    fn move_view_down_unchecked(&mut self) {
        self.view.row += 1;
    }

    fn move_view_left_unchecked(&mut self) {
        self.view.col -= 1;
    }

    fn move_view_right_unchecked(&mut self) {
        self.view.col += 1;
    }

    fn is_cursor_at_view_top(&self) -> bool {
        self.cursor.row == self.view.row
    }

    fn is_cursor_at_view_bottom(&self) -> bool {
        self.cursor.row == self.view.last_row()
    }

    fn is_cursor_at_view_left(&self) -> bool {
        self.cursor.col == self.view.col
    }

    fn is_cursor_at_view_right(&self) -> bool {
        self.cursor.col == self.view.last_col()
    }

    fn is_cursor_at_buffer_top(&self) -> bool {
        self.cursor.row == 0
    }

    fn is_cursor_at_buffer_bottom(&self) -> bool {
        self.cursor.row == self.rendered_buffer.last_line()
    }

    fn is_cursor_at_line_start(&self) -> bool {
        self.cursor.col == 0
    }

    fn is_cursor_at_eol_col(&self) -> bool {
        self.cursor.col == self.rendered_buffer.eol_col(self.cursor.row)
    }

    fn is_cursor_past_eol(&self) -> bool {
        self.cursor.col > self.rendered_buffer.eol_col(self.cursor.row)
    }
}
