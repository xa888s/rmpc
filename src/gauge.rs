use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    style::Style,
    widgets::{Block, Widget},
};

pub struct SimpleGauge<'a> {
    block: Option<Block<'a>>,
    ratio: f64,
    style: Style,
    gauge_style: Style,
}

impl<'a> Default for SimpleGauge<'a> {
    fn default() -> SimpleGauge<'a> {
        SimpleGauge {
            block: None,
            ratio: 0.0,
            style: Style::default(),
            gauge_style: Style::default(),
        }
    }
}

impl<'a> SimpleGauge<'a> {
    pub fn block(mut self, block: Block<'a>) -> SimpleGauge<'a> {
        self.block = Some(block);
        self
    }

    pub fn percent(mut self, percent: u16) -> SimpleGauge<'a> {
        assert!(
            percent <= 100,
            "Percentage should be between 0 and 100 inclusively."
        );
        self.ratio = f64::from(percent) / 100.0;
        self
    }
}

impl<'a> Widget for SimpleGauge<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let gauge_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        buf.set_style(gauge_area, self.gauge_style);
        if gauge_area.height < 1 {
            return;
        }

        let width = (f64::from(gauge_area.width) * self.ratio).round() as u16;
        let end = gauge_area.left() + width;
        for y in gauge_area.top()..gauge_area.bottom() {
            // Gauge
            for x in gauge_area.left()..end {
                buf.get_mut(x, y).set_symbol("=");
            }
            buf.get_mut(end, y).set_symbol(">");

            // Fix colors
            for x in gauge_area.left()..end {
                buf.get_mut(x, y)
                    .set_fg(self.gauge_style.bg.unwrap_or(Color::Reset))
                    .set_bg(self.gauge_style.fg.unwrap_or(Color::Reset));
            }
        }
    }
}
