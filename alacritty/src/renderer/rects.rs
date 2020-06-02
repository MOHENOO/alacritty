use crossfont::Metrics;

use alacritty_terminal::index::{Column, Point};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::color::Rgb;
use alacritty_terminal::term::SizeInfo;
use alacritty_terminal::text_run::TextRun;

#[derive(Debug, Copy, Clone)]
pub struct RenderRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: Rgb,
    pub alpha: f32,
}

impl RenderRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32, color: Rgb, alpha: f32) -> Self {
        RenderRect { x, y, width, height, color, alpha }
    }

    #[inline]
    pub fn from_text_run(
        text_run: &TextRun,
        descent: f32,
        position: f32,
        thickness: f32,
        size: &SizeInfo,
    ) -> Self {
        let start_point = text_run.start_point();
        let start_x = start_point.col.0 as f32 * size.cell_width();
        let end_x = (text_run.end_point().col.0 + 1) as f32 * size.cell_width();
        let width = end_x - start_x;

        let line_bottom = (start_point.line.0 + 1) as f32 * size.cell_height();
        let baseline = line_bottom + descent;

        // Make sure lines are always visible.
        let height = thickness.max(1.);

        let mut y = (baseline - position - height / 2.).ceil();
        let max_y = line_bottom - height;
        y = y.min(max_y);

        Self {
            x: start_x + size.padding_x(),
            y: y + size.padding_y(),
            width,
            height,
            color: text_run.fg,
            alpha: 1.,
        }
    }

    pub fn all_from_text_run(
        text_run: &TextRun,
        metrics: &crossfont::Metrics,
        size_info: &SizeInfo,
    ) -> Vec<Self> {
        let mut rects = vec![];
        if text_run.flags.contains(Flags::UNDERLINE) {
            rects.push(Self::from_text_run(
                text_run,
                metrics.descent,
                metrics.underline_position,
                metrics.underline_thickness,
                size_info,
            ));
        }
        if text_run.flags.contains(Flags::STRIKEOUT) {
            rects.push(Self::from_text_run(
                text_run,
                metrics.descent,
                metrics.strikeout_position,
                metrics.strikeout_thickness,
                size_info,
            ));
        }
        if text_run.flags.contains(Flags::DOUBLE_UNDERLINE) {
            let top_pos = 0.25 * metrics.descent;
            let bottom_pos = 0.75 * metrics.descent;
            rects.push(Self::from_text_run(
                text_run,
                metrics.descent,
                top_pos,
                metrics.underline_thickness,
                size_info,
            ));
            rects.push(Self::from_text_run(
                text_run,
                metrics.descent,
                bottom_pos,
                metrics.underline_thickness,
                size_info,
            ));
        }
        rects
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RenderLine {
    pub start: Point,
    pub end: Point,
    pub color: Rgb,
}

impl RenderLine {
    pub fn rects(&self, flag: Flags, metrics: &Metrics, size: &SizeInfo) -> Vec<RenderRect> {
        let mut rects = Vec::new();

        let mut start = self.start;
        while start.line < self.end.line {
            let end = Point::new(start.line, size.cols() - 1);
            Self::push_rects(&mut rects, metrics, size, flag, start, end, self.color);
            start = Point::new(start.line + 1, Column(0));
        }
        Self::push_rects(&mut rects, metrics, size, flag, start, self.end, self.color);

        rects
    }

    /// Push all rects required to draw the cell's line.
    fn push_rects(
        rects: &mut Vec<RenderRect>,
        metrics: &Metrics,
        size: &SizeInfo,
        flag: Flags,
        start: Point,
        end: Point,
        color: Rgb,
    ) {
        let (position, thickness) = match flag {
            Flags::DOUBLE_UNDERLINE => {
                // Position underlines so each one has 50% of descent available.
                let top_pos = 0.25 * metrics.descent;
                let bottom_pos = 0.75 * metrics.descent;

                rects.push(Self::create_rect(
                    size,
                    metrics.descent,
                    start,
                    end,
                    top_pos,
                    metrics.underline_thickness,
                    color,
                ));

                (bottom_pos, metrics.underline_thickness)
            },
            Flags::UNDERLINE => (metrics.underline_position, metrics.underline_thickness),
            Flags::STRIKEOUT => (metrics.strikeout_position, metrics.strikeout_thickness),
            _ => unimplemented!("Invalid flag for cell line drawing specified"),
        };

        rects.push(Self::create_rect(
            size,
            metrics.descent,
            start,
            end,
            position,
            thickness,
            color,
        ));
    }

    /// Create a line's rect at a position relative to the baseline.
    fn create_rect(
        size: &SizeInfo,
        descent: f32,
        start: Point,
        end: Point,
        position: f32,
        mut thickness: f32,
        color: Rgb,
    ) -> RenderRect {
        let start_x = start.col.0 as f32 * size.cell_width();
        let end_x = (end.col.0 + 1) as f32 * size.cell_width();
        let width = end_x - start_x;

        // Make sure lines are always visible.
        thickness = thickness.max(1.);

        let line_bottom = (start.line.0 as f32 + 1.) * size.cell_height();
        let baseline = line_bottom + descent;

        let mut y = (baseline - position - thickness / 2.).ceil();
        let max_y = line_bottom - thickness;
        if y > max_y {
            y = max_y;
        }

        RenderRect::new(
            start_x + size.padding_x(),
            y + size.padding_y(),
            width,
            thickness,
            color,
            1.,
        )
    }
}
