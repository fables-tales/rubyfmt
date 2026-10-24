use crate::types::{LineNumber, SourceOffset};

/// Tracks the formatter's current source line and resolves byte offsets to lines.
///
/// The cursor is only an optimization for `line_number_for_offset`; it may point
/// ahead of `current_line` after a lookahead query. Keeping it in the same type
/// as `current_line` ensures updates to the current line also rebase the cursor.
#[derive(Debug, Clone)]
pub struct LineTracker<'src> {
    line_starts: &'src [SourceOffset],
    current_line: LineNumber,
    cursor: usize,
}

impl<'src> LineTracker<'src> {
    pub fn new(line_starts: &'src [SourceOffset]) -> Self {
        Self {
            line_starts,
            current_line: 0,
            cursor: 0,
        }
    }

    pub fn current_line(&self) -> LineNumber {
        self.current_line
    }

    pub fn set_current_line(&mut self, line_number: LineNumber) {
        self.current_line = line_number;
        let line_index = (line_number.max(1) - 1) as usize;
        self.cursor = line_index.min(self.line_starts.len().saturating_sub(1));
    }

    pub fn advance_line_by(&mut self, line_count: LineNumber) {
        self.set_current_line(self.current_line + line_count);
    }

    pub fn current_line_end_offset(&self) -> SourceOffset {
        self.line_starts
            .get(self.current_line as usize)
            .copied()
            .unwrap_or(SourceOffset::MAX)
    }

    pub fn line_number_for_offset(&mut self, source_offset: SourceOffset) -> LineNumber {
        (self.line_index_for_offset(source_offset) + 1) as LineNumber
    }

    fn line_index_for_offset(&mut self, source_offset: SourceOffset) -> usize {
        if self.line_starts.is_empty() {
            return 0;
        }

        let offsets = self.line_starts;
        let last = offsets.len() - 1;
        let mut cursor = self.cursor.min(last);

        if offsets[cursor] <= source_offset
            && (cursor == last || source_offset < offsets[cursor + 1])
        {
            self.cursor = cursor;
            return cursor;
        }

        // Formatting is mostly source-order, so advancing a few lines is usually
        // cheaper than restarting a binary search.
        if source_offset >= offsets[cursor] {
            for _ in 0..4 {
                if cursor == last {
                    break;
                }
                cursor += 1;
                if cursor == last || source_offset < offsets[cursor + 1] {
                    self.cursor = cursor;
                    return cursor;
                }
            }
        }

        let index = offsets
            .partition_point(|&line_start| line_start <= source_offset)
            .saturating_sub(1)
            .min(last);
        self.cursor = index;
        index
    }
}

#[cfg(test)]
mod tests {
    use super::LineTracker;

    #[test]
    fn offset_lookahead_does_not_move_current_line() {
        let line_starts = [0, 10, 20, 30];
        let mut lines = LineTracker::new(&line_starts);
        lines.set_current_line(2);

        assert_eq!(lines.current_line_end_offset(), 20);
        assert_eq!(lines.line_number_for_offset(25), 3);
        assert_eq!(lines.current_line(), 2);

        lines.advance_line_by(1);

        assert_eq!(lines.current_line(), 3);
        assert_eq!(lines.current_line_end_offset(), 30);
        assert_eq!(lines.line_number_for_offset(5), 1);
        assert_eq!(lines.current_line(), 3);
    }
}
