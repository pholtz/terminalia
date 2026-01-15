use ratatui::{Frame, text::Text, widgets::Paragraph};
use specs::prelude::*;

pub fn render_trading(ecs: &mut World, frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new(Text::from("Heyo!")),
        frame.area()
    );
}
