use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Stylize},
    symbols::border,
    text::Text,
    widgets::{Block, Borders, Padding, Paragraph},
};

const TITLE: &str = "
████████╗███████╗██████╗ ███╗   ███╗██╗███╗   ██╗ █████╗ ██╗     ██╗ █████╗ 
╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██║████╗  ██║██╔══██╗██║     ██║██╔══██╗
   ██║   █████╗  ██████╔╝██╔████╔██║██║██╔██╗ ██║███████║██║     ██║███████║
   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║██║╚██╗██║██╔══██║██║     ██║██╔══██║
   ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║██║ ╚████║██║  ██║███████╗██║██║  ██║
   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚═╝╚═╝  ╚═╝
                                                                            
";

/**
 * Renders the menu for the game.
 *
 * Should consist of a border and a couple selectable menu items for now.
 * Each one will change the main screen state.
 */
pub fn render_menu(frame: &mut Frame<'_>, menu_index: u8) {
    let menu = Block::default()
        .borders(Borders::all())
        .padding(Padding::symmetric(5, 6))
        .inner(frame.area());
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Fill(1),
            Constraint::Percentage(50),
            Constraint::Fill(1),
        ])
        .split(menu);
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Fill(1),
            Constraint::Percentage(25),
            Constraint::Fill(1),
        ])
        .split(vertical_layout[1]);

    /*
     * Render the game title at the top middle of the layout
     */
    frame.render_widget(
        Paragraph::new(Text::from(TITLE)).centered(),
        vertical_layout[0],
    );

    /*
     * Render the menu buttons inside the middle middle of the layout
     */
    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3), Constraint::Length(3)])
        .split(horizontal_layout[1]);
    frame.render_widget(
        Paragraph::new(Text::from("New Game"))
            .centered()
            .bg(if menu_index == 0 {
                Color::Cyan
            } else {
                Color::Black
            })
            .block(Block::bordered().border_set(border::THICK)),
        menu_layout[0],
    );
    frame.render_widget(
        Paragraph::new(Text::from("Quit"))
            .centered()
            .bg(if menu_index == 1 {
                Color::Cyan
            } else {
                Color::Black
            })
            .block(Block::bordered().border_set(border::THICK)),
        menu_layout[1],
    );
}
