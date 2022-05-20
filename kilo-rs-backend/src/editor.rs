use std::cmp;
use std::ops::Range;

use crate::core::{Buffer, Location, Span};
use crate::view::{rendering::RenderedBuffer, ViewGeometry};

use anyhow::Result;
use syntect::highlighting::Style;

pub struct Editor {
    buffer: Buffer,
    rendered_buffer: RenderedBuffer,
    cursor: Location,
    view: ViewGeometry,
    search_state: Option<SearchState>,
}

struct SearchState {
    initial_cursor: Location,
    initial_view: ViewGeometry,
    pattern: Option<String>,
    forward: bool,
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
            search_state: None,
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

    pub fn get_view_contents(
        &self,
    ) -> (
        impl Iterator<Item = String>,
        Option<impl Iterator<Item = Vec<(Style, Range<usize>)>>>,
    ) {
        let ViewGeometry {
            line,
            col,
            width,
            height,
        } = self.view;
        // TODO: Reimplement this
        // let filler = once("~").chain(repeat(" ")).take(width).collect();
        // self.rendered_buffer
        //     .get_view(line, col, width, height)
        //     .into_iter()
        //     .chain(repeat(filler))
        //     .take(height)
        let tuple = self.rendered_buffer.get_view(line, col, width, height);
        (tuple.0.into_iter(), tuple.1.map(|vec| vec.into_iter()))
    }

    pub fn get_file_name(&self) -> Option<&String> {
        self.buffer.file_path()
    }

    pub fn is_buffer_dirty(&self) -> bool {
        self.buffer.is_dirty()
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
        let should_move_view = self.is_cursor_at_view_bottom();

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
            self.move_cursor_to_line_start();
        } else {
            self.buffer.split_line(self.cursor);
            self.rendered_buffer
                .update_line(self.cursor.line, &self.buffer);
            self.rendered_buffer
                .insert_line(self.cursor.line + 1, &self.buffer);
            self.move_cursor_down_unchecked();
            self.move_cursor_to_line_start();
        }

        if should_move_view {
            self.move_view_down_unchecked();
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

    pub fn is_search_mode_active(&self) -> bool {
        self.search_state.is_some()
    }

    pub fn start_search(&mut self) {
        self.search_state = Some(SearchState {
            initial_cursor: self.cursor,
            initial_view: self.view.clone(),
            pattern: None,
            forward: true,
        });
    }

    pub fn set_search_pattern(&mut self, pattern: &str) {
        self.search_state.as_mut().unwrap().pattern = Some(String::from(pattern));

        let state = self.search_state.as_ref().unwrap();
        let cursor = state.initial_cursor;
        let forward = state.forward;
        let pattern = state.pattern.as_ref().unwrap();

        if let Some(Span { start, .. }) = self.buffer.find(pattern, forward, cursor) {
            self.move_cursor_to_location(start);
        }
    }

    pub fn set_search_direction(&mut self, forward: bool) {
        self.search_state.as_mut().unwrap().forward = forward;
    }

    pub fn next_search_result(&mut self) {
        let state = self.search_state.as_ref().unwrap();
        let forward = state.forward;
        let pattern = state.pattern.as_ref().unwrap();

        if let Some(Span { start, .. }) = self.buffer.find(pattern, forward, self.cursor) {
            self.move_cursor_to_location(start);
        }
    }

    pub fn finish_search(&mut self) {
        self.search_state = None;
    }

    pub fn cancel_search(&mut self) {
        let search_state = self.search_state.take().unwrap();

        self.cursor = search_state.initial_cursor;
        self.view = search_state.initial_view;
    }

    fn move_cursor_to_location(&mut self, location: Location) {
        self.cursor = location;
        if self.cursor.line < self.view.line || self.cursor.line > self.view.last_line() {
            self.view.line = std::cmp::min(self.cursor.line, self.bottom_most_view_pos());
        }
        if self.cursor.col < self.view.col || self.cursor.col > self.view.last_col() {
            self.view.col = self.cursor.col;
        }
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
        eprintln!("{}", self.rendered_buffer.eol_col(self.cursor.line));
        self.cursor.col == self.rendered_buffer.eol_col(self.cursor.line)
    }

    fn is_cursor_past_eol(&self) -> bool {
        self.cursor.col > self.rendered_buffer.eol_col(self.cursor.line)
    }
}
