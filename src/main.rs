use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{
        canvas::{Canvas, Rectangle},
        Block, Borders, Paragraph,
    },
    Frame,
};
use std::time::{Duration, Instant};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const CELL_CHARS: &str = "    "; // Four spaces for a wider block
const TICK_RATE: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    Filled(Color),
}

#[derive(Clone, Copy, PartialEq)]
enum TetrominoType {
    I,
    O,
    T,
    L,
    J,
    S,
    Z,
}

impl TetrominoType {
    fn color(&self) -> Color {
        match self {
            TetrominoType::I => Color::Cyan,
            TetrominoType::O => Color::Yellow,
            TetrominoType::T => Color::Magenta,
            TetrominoType::L => Color::White,
            TetrominoType::J => Color::Blue,
            TetrominoType::S => Color::Green,
            TetrominoType::Z => Color::Red,
        }
    }

    fn shape(&self) -> Vec<Vec<bool>> {
        match self {
            TetrominoType::I => vec![
                vec![true, true, true, true],
                vec![false, false, false, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ],
            TetrominoType::O => vec![vec![true, true], vec![true, true]],
            TetrominoType::T => vec![
                vec![false, true, false],
                vec![true, true, true],
                vec![false, false, false],
            ],
            TetrominoType::L => vec![
                vec![false, false, true],
                vec![true, true, true],
                vec![false, false, false],
            ],
            TetrominoType::J => vec![
                vec![true, false, false],
                vec![true, true, true],
                vec![false, false, false],
            ],
            TetrominoType::S => vec![
                vec![false, true, true],
                vec![true, true, false],
                vec![false, false, false],
            ],
            TetrominoType::Z => vec![
                vec![true, true, false],
                vec![false, true, true],
                vec![false, false, false],
            ],
        }
    }
}

struct Tetromino {
    piece_type: TetrominoType,
    shape: Vec<Vec<bool>>,
    x: i32,
    y: i32,
}

impl Tetromino {
    fn new_random() -> Self {
        use rand::seq::SliceRandom;

        let piece_types = [
            TetrominoType::I,
            TetrominoType::O,
            TetrominoType::T,
            TetrominoType::L,
            TetrominoType::J,
            TetrominoType::S,
            TetrominoType::Z,
        ];

        let piece_type = *piece_types.choose(&mut rand::thread_rng()).unwrap();
        let shape = piece_type.shape();
        let width = shape[0].len() as i32;

        Tetromino {
            piece_type,
            shape,
            x: (BOARD_WIDTH as i32 - width) / 2,
            y: 0,
        }
    }

    fn rotate_clockwise(&self) -> Vec<Vec<bool>> {
        let n = self.shape.len();
        let mut rotated = vec![vec![false; n]; n];

        for i in 0..n {
            for j in 0..n {
                rotated[j][n - 1 - i] = self.shape[i][j];
            }
        }

        rotated
    }

    fn color(&self) -> Color {
        self.piece_type.color()
    }
}

struct Game {
    board: Vec<Vec<Cell>>,
    current_piece: Tetromino,
    last_tick: Instant,
    game_over: bool,
    score: u32,
}

impl Game {
    fn rotate_piece(&mut self) {
        let rotated_shape = self.current_piece.rotate_clockwise();

        // Try normal rotation
        if self.is_valid_position(&rotated_shape, self.current_piece.x, self.current_piece.y) {
            self.current_piece.shape = rotated_shape;
            return;
        }

        // Wall kick: try shifting left if rotation fails
        if self.is_valid_position(
            &rotated_shape,
            self.current_piece.x - 1,
            self.current_piece.y,
        ) {
            self.current_piece.shape = rotated_shape;
            self.current_piece.x -= 1;
            return;
        }

        // Wall kick: try shifting right if rotation fails
        if self.is_valid_position(
            &rotated_shape,
            self.current_piece.x + 1,
            self.current_piece.y,
        ) {
            self.current_piece.shape = rotated_shape;
            self.current_piece.x += 1;
            return;
        }

        // If all attempts fail, the rotation is not performed
    }

    fn new() -> Self {
        Game {
            board: vec![vec![Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: Tetromino::new_random(),
            last_tick: Instant::now(),
            game_over: false,
            score: 0,
        }
    }

    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;

        // Check each line from bottom to top
        let mut y = BOARD_HEIGHT - 1;
        while y > 0 {
            // Check if current line is full
            if self.board[y]
                .iter()
                .all(|cell| matches!(cell, Cell::Filled(_)))
            {
                // Move all lines above down by one
                for row in (1..=y).rev() {
                    self.board[row] = self.board[row - 1].clone();
                }
                // Create new empty line at top
                self.board[0] = vec![Cell::Empty; BOARD_WIDTH];
                lines_cleared += 1;
            } else {
                y -= 1;
            }
        }

        match lines_cleared {
            1 => self.score += 100,
            2 => self.score += 300,
            3 => self.score += 500,
            4 => self.score += 800,
            _ => (),
        }
    }

    fn tick(&mut self) {
        if self.game_over {
            return;
        }

        if !self.move_piece(0, 1) {
            self.freeze_piece();
            self.clear_lines();
            self.spawn_new_piece();
        }
    }

    fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        let new_x = self.current_piece.x + dx;
        let new_y = self.current_piece.y + dy;

        if self.is_valid_position(&self.current_piece.shape, new_x, new_y) {
            self.current_piece.x = new_x;
            self.current_piece.y = new_y;
            true
        } else {
            false
        }
    }

    // Update is_valid_position to check for Cell::Empty
    fn is_valid_position(&self, shape: &Vec<Vec<bool>>, x: i32, y: i32) -> bool {
        for (row_idx, row) in shape.iter().enumerate() {
            for (col_idx, &is_filled) in row.iter().enumerate() {
                if is_filled {
                    let board_x = x + col_idx as i32;
                    let board_y = y + row_idx as i32;

                    if board_x < 0
                        || board_x >= BOARD_WIDTH as i32
                        || board_y >= BOARD_HEIGHT as i32
                    {
                        return false;
                    }

                    if board_y >= 0 {
                        match self.board[board_y as usize][board_x as usize] {
                            Cell::Empty => {}
                            Cell::Filled(_) => return false,
                        }
                    }
                }
            }
        }
        true
    }

    fn freeze_piece(&mut self) {
        let color = self.current_piece.color();
        for (row_idx, row) in self.current_piece.shape.iter().enumerate() {
            for (col_idx, &is_filled) in row.iter().enumerate() {
                if is_filled {
                    let board_x = self.current_piece.x + col_idx as i32;
                    let board_y = self.current_piece.y + row_idx as i32;
                    if board_y >= 0 && board_y < BOARD_HEIGHT as i32 {
                        self.board[board_y as usize][board_x as usize] = Cell::Filled(color);
                    }
                }
            }
        }
    }

    fn spawn_new_piece(&mut self) {
        self.current_piece = Tetromino::new_random();

        // Check if the new piece can be placed at spawn position
        if !self.is_valid_position(
            &self.current_piece.shape,
            self.current_piece.x,
            self.current_piece.y,
        ) {
            self.game_over = true;
        }
    }
}

fn main() {
    let mut terminal = ratatui::init();
    let mut game = Game::new();

    loop {
        terminal
            .draw(|f| draw(f, &game))
            .expect("failed to draw frame");

        if game.last_tick.elapsed() >= TICK_RATE {
            game.tick();
            game.last_tick = Instant::now();
        }

        if event::poll(Duration::from_millis(50)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => {
                        game.move_piece(-1, 0);
                    }
                    KeyCode::Right => {
                        game.move_piece(1, 0);
                    }
                    KeyCode::Down => {
                        game.move_piece(0, 1);
                    }
                    KeyCode::Up => {
                        game.rotate_piece();
                    }
                    _ => {}
                }
            }
        }
    }
    ratatui::restore()
}

fn draw(frame: &mut Frame, game: &Game) {
    // Create the main layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(frame.area());

    draw_game_board(frame, game, chunks[0]);
    draw_side_panel(frame, game, chunks[1]);
}

fn draw_game_board(frame: &mut Frame, game: &Game, area: Rect) {
    // Create a temporary board with current piece
    let mut display_board = game.board.clone();

    // Add current piece to display board
    for (row_idx, row) in game.current_piece.shape.iter().enumerate() {
        for (col_idx, &is_filled) in row.iter().enumerate() {
            if is_filled {
                let board_x = game.current_piece.x + col_idx as i32;
                let board_y = game.current_piece.y + row_idx as i32;
                if board_y >= 0
                    && board_y < BOARD_HEIGHT as i32
                    && board_x >= 0
                    && board_x < BOARD_WIDTH as i32
                {
                    display_board[board_y as usize][board_x as usize] =
                        Cell::Filled(game.current_piece.color());
                }
            }
        }
    }

    // Scale vertically by repeating each row
    let vertical_scale = 2; // Each cell is 2 rows high
    let mut scaled_rows = Vec::with_capacity(BOARD_HEIGHT * vertical_scale);

    for y in 0..BOARD_HEIGHT {
        let row_spans: Vec<ratatui::text::Span> = display_board[y]
            .iter()
            .map(|cell| match cell {
                Cell::Empty => {
                    ratatui::text::Span::styled(CELL_CHARS, Style::default().bg(Color::Gray))
                }
                Cell::Filled(color) => {
                    ratatui::text::Span::styled(CELL_CHARS, Style::default().bg(*color))
                }
            })
            .collect();

        let line = ratatui::text::Line::from(row_spans);
        // Add each row multiple times for vertical scaling
        for _ in 0..vertical_scale {
            scaled_rows.push(line.clone());
        }
    }

    let board_widget = Paragraph::new(scaled_rows).block(Block::default().title("Tetris"));

    // Calculate the maximum space we can use while maintaining aspect ratio
    let available_height = area.height as usize - 2; // -2 for borders
    let available_width = (area.width as usize - 2) / CELL_CHARS.len(); // Account for cell width

    let height_ratio = available_height as f32 / (BOARD_HEIGHT * vertical_scale) as f32;
    let width_ratio = available_width as f32 / BOARD_WIDTH as f32;

    // Use the smaller ratio to maintain aspect ratio
    let ratio = height_ratio.min(width_ratio);

    let used_height = (BOARD_HEIGHT * vertical_scale) as f32 * ratio;
    let used_width = (BOARD_WIDTH * CELL_CHARS.len()) as f32 * ratio + 2.0; // +2 for borders

    // Center the board in the available space
    let vertical_padding = ((area.height as f32 - used_height) / 2.0).floor() as u16;
    let horizontal_padding = ((area.width as f32 - used_width) / 2.0).floor() as u16;

    let centered_area = Rect {
        x: area.x + horizontal_padding,
        y: area.y + vertical_padding,
        width: used_width as u16,
        height: used_height as u16 + 2, // +2 for borders
    };

    frame.render_widget(board_widget, centered_area);
}

fn draw_side_panel(frame: &mut Frame, game: &Game, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Score
            Constraint::Length(6), // Next piece
            Constraint::Min(0),    // Controls
        ])
        .split(area);

    // Score
    let score_text = format!("Score: {}", game.score);
    let score = Paragraph::new(score_text)
        .block(Block::default().borders(Borders::ALL).title("Score"))
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(score, chunks[0]);

    // Controls help
    let controls = vec![
        "Controls:",
        "←/→: Move",
        "↑: Rotate",
        "↓: Soft Drop",
        "Q: Quit",
    ]
    .join("\n");

    let controls_widget = Paragraph::new(controls)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(controls_widget, chunks[2]);
}
