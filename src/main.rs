use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const TICK_RATE: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    Filled,
}

struct Tetromino {
    shape: Vec<Vec<bool>>,
    x: i32,
    y: i32,
}

struct Game {
    board: Vec<Vec<Cell>>,
    current_piece: Tetromino,
    last_tick: Instant,
    game_over: bool,
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
        }
    }

    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;

        // Check each line from bottom to top
        let mut y = BOARD_HEIGHT - 1;
        while y > 0 {
            // Check if current line is full
            if self.board[y].iter().all(|&cell| cell == Cell::Filled) {
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

        // Here you could add scoring based on lines_cleared
        // Traditional scoring is:
        // 1 line = 100 points
        // 2 lines = 300 points
        // 3 lines = 500 points
        // 4 lines = 800 points (Tetris!)
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

                    if board_y >= 0
                        && self.board[board_y as usize][board_x as usize] == Cell::Filled
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn freeze_piece(&mut self) {
        for (row_idx, row) in self.current_piece.shape.iter().enumerate() {
            for (col_idx, &is_filled) in row.iter().enumerate() {
                if is_filled {
                    let board_x = self.current_piece.x + col_idx as i32;
                    let board_y = self.current_piece.y + row_idx as i32;
                    if board_y >= 0 && board_y < BOARD_HEIGHT as i32 {
                        self.board[board_y as usize][board_x as usize] = Cell::Filled;
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

impl Tetromino {
    fn new_random() -> Self {
        use rand::seq::SliceRandom;

        // All 7 standard tetromino shapes
        let shapes = vec![
            // I piece
            vec![
                vec![true, true, true, true],
                vec![false, false, false, false],
                vec![false, false, false, false],
                vec![false, false, false, false],
            ],
            // O piece
            vec![vec![true, true], vec![true, true]],
            // T piece
            vec![
                vec![false, true, false],
                vec![true, true, true],
                vec![false, false, false],
            ],
            // L piece
            vec![
                vec![false, false, true],
                vec![true, true, true],
                vec![false, false, false],
            ],
            // J piece
            vec![
                vec![true, false, false],
                vec![true, true, true],
                vec![false, false, false],
            ],
            // S piece
            vec![
                vec![false, true, true],
                vec![true, true, false],
                vec![false, false, false],
            ],
            // Z piece
            vec![
                vec![true, true, false],
                vec![false, true, true],
                vec![false, false, false],
            ],
        ];

        let shape = shapes.choose(&mut rand::thread_rng()).unwrap().clone();
        let width = shape[0].len() as i32;

        Tetromino {
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
    // Create a temporary board that includes both frozen pieces and current piece
    let mut display_board = game.board.clone();

    // Draw current piece onto the temporary board
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
                    display_board[board_y as usize][board_x as usize] = Cell::Filled;
                }
            }
        }
    }

    // Convert the board to a displayable string
    let board_display: String = display_board
        .iter()
        .map(|row| {
            row.iter()
                .map(|&cell| match cell {
                    Cell::Empty => '.',
                    Cell::Filled => '#',
                })
                .collect::<String>()
                + "\n"
        })
        .collect();

    let paragraph =
        Paragraph::new(board_display).block(Block::default().borders(Borders::ALL).title("Tetris"));

    frame.render_widget(paragraph, frame.size());
}
