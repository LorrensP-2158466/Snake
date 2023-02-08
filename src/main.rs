use rand::{thread_rng, Rng};
use std::collections::VecDeque;
use std::io::{stdin, stdout, Write};
use std::{thread, time};
use termion::event::*;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor};

// top left corner is (0, 0)
// bottom right corner is (max_height, max_width)
const SPACE: Char = Char {
    info: b' ',
    col: Colour::Black,
};
const FOOD: Char = Char {
    info: b'@',
    col: Colour::Red,
};
const SNAKE_PART: Char = Char {
    info: b'o',
    col: Colour::Green,
};
const SNAKE_HEAD: Char = Char {
    info: b'0',
    col: Colour::Green,
};
const LINE: Char = Char {
    info: b'-',
    col: Colour::White,
};
const L_TOOTH: Char = Char {
    info: b'<',
    col: Colour::White,
};
const R_TOOTH: Char = Char {
    info: b'>',
    col: Colour::White,
};
const V: Char = Char {
    info: b'v',
    col: Colour::White,
};
const CARET: Char = Char {
    info: b'^',
    col: Colour::White,
};

const UPPER_LINE: [Char; WIDTH] = [V; WIDTH];
const LOWER_LINE: [Char; WIDTH] = [CARET; WIDTH];
const HEIGHT: usize = 14;
const WIDTH: usize = 30;
const FPS: usize = 15;

#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd)]
enum Colour {
    Green,
    Red,
    Black,
    White,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd)]
struct Char {
    pub info: u8,
    pub col: Colour,
}

impl Char {
    pub fn new(info: u8, col: Colour) -> Self {
        Self { info, col }
    }
}

struct Game<W: Write> {
    pub snake: Snake,
    pub food: Pos,
    pub score: i32,
    pub state: bool, // true for playing, false if snake died
    writer: W,
    buff: [[Char; WIDTH]; HEIGHT],
}

impl<W: Write> Game<W> {
    pub fn new(w: W) -> Self {
        Self {
            snake: Snake::new(Pos { x: 14, y: 7 }),
            food: Pos {
                x: thread_rng().gen_range(1..WIDTH - 1),
                y: thread_rng().gen_range(1..HEIGHT - 1),
            },
            score: 0,
            state: true,
            writer: w,
            buff: [[SPACE; WIDTH]; HEIGHT],
        }
    }
    pub fn start(&mut self) {
        use termion::event::Key::*;
        let mut running = true;
        let mut stdin = stdin().lock().keys();
        while running {
            self.play();
            let b = stdin.next().unwrap().unwrap();
            match b {
                Char('Y') | Char('y') => {
                    self.reset();
                    continue;
                }
                Char('N') | Char('n') => {
                    running = false;
                }
                _ => {}
            }
        }
    }

    pub fn play(&mut self) {
        use termion::event::Key::*;
        write!(self.writer, "{}{}", cursor::Hide, clear::All).unwrap();
        self.draw_game();
        let mut user_input = termion::async_stdin().events();
        loop {
            if let Some(event) = user_input.next() {
                match event {
                    Ok(Event::Key(key)) => match key {
                        Char('q') | Left => self.snake.set_dir(Direction::Left),
                        Char('s') | Down => self.snake.set_dir(Direction::Down),
                        Char('d') | Right => self.snake.set_dir(Direction::Right),
                        Char('z') | Up => self.snake.set_dir(Direction::Up),
                        Esc => break,
                        _ => {}
                    },
                    _ => {}
                }
            }

            if !self.snake_alive() {
                self.state = false;
                break;
            }
            self.snake.go();
            if self.snake.head() == self.food {
                self.snake_eats();
            }
            self.draw_game();
            thread::sleep(time::Duration::from_millis(10 * FPS as u64))
        }
        self.exit_screen();
    }

    fn reset(&mut self) {
        self.buffer_reset();
        self.snake = Snake::new(Pos { x: 14, y: 7 });
        self.food = Pos {
            x: thread_rng().gen_range(1..WIDTH - 1),
            y: thread_rng().gen_range(1..HEIGHT - 1),
        };
        self.score = 0;
        self.state = true;
    }

    fn snake_eats(&mut self) {
        self.snake.grow();
        self.food = Pos {
            x: thread_rng().gen_range(1..WIDTH - 1),
            y: thread_rng().gen_range(1..HEIGHT - 1),
        };
        self.score += 1;
    }

    fn snake_alive(&self) -> bool {
        let canibalism = self.snake.ate_itself();
        let snake_bumbed = !self
            .snake
            .head()
            .inbound(Pos::new(0, 0), Pos::new(WIDTH - 1, HEIGHT - 1));
        return !canibalism && !snake_bumbed;
    }

    fn draw_game(&mut self) {
        // insert snake and food in buffer -> draw to screen
        write!(self.writer, "{}", cursor::Goto(1, 1)).unwrap();
        write!(self.writer, "Score: {}\r\n", self.score).unwrap();
        // draw the lines
        self.buffer_reset();
        self.draw_field();
        self.draw_snake();
        self.draw_food();
        self.draw_buffer();
    }
    fn buffer_reset(&mut self) {
        self.buff = [[SPACE; WIDTH]; HEIGHT];
    }
    fn draw_field(&mut self) {
        self.buff[0] = UPPER_LINE;
        self.buff[HEIGHT - 1] = LOWER_LINE;
        for i in 1..HEIGHT {
            self.buff[i][0] = R_TOOTH;
            self.buff[i][WIDTH - 1] = L_TOOTH;
        }
    }
    fn draw_buffer(&mut self) {
        for &line in self.buff.iter() {
            for &c in line.iter() {
                match c.col {
                    Colour::Black => {
                        write!(self.writer, "{}", color::Fg(color::Black)).unwrap();
                    }
                    Colour::White => {
                        write!(self.writer, "{}", color::Fg(color::White)).unwrap();
                    }
                    Colour::Green => {
                        write!(self.writer, "{}", color::Fg(color::Green)).unwrap();
                    }
                    Colour::Red => {
                        write!(self.writer, "{}", color::Fg(color::Red)).unwrap();
                    }
                }
                self.writer.write(&[c.info]).unwrap();
            }
            write!(self.writer, "{}", color::Fg(color::Reset)).unwrap();
            self.writer.write(b"\r\n").unwrap();
        }
        self.writer.flush().unwrap();
    }
    fn draw_food(&mut self) {
        self.buff[self.food.y][self.food.x] = FOOD;
    }

    fn draw_snake(&mut self) {
        for p in self.snake.body.iter() {
            self.buff[p.y][p.x] = SNAKE_PART;
        }
        let head = self.snake.head();
        self.buff[head.y][head.x] = SNAKE_HEAD;
    }

    fn exit_screen(&mut self) {
        write!(self.writer, "{}", cursor::Goto(1, 1)).unwrap();
        write!(self.writer, "Score: {}\r\n", self.score).unwrap();
        self.buffer_reset();
        self.draw_field();
        for x in 10..20 {
            self.buff[4][x] = LINE;
            self.buff[8][x] = LINE;
        }
        let mut text_it = self.buff[5].iter_mut().skip(11);
        for &el in b"You Died".iter() {
            *text_it.next().unwrap() = Char::new(el, Colour::White);
        }
        let mut no_it = self.buff[7].iter_mut().skip(9);
        for &el in b"Again?: (Y/N)".iter() {
            *no_it.next().unwrap() = Char::new(el, Colour::White);
        }
        self.draw_buffer();
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd)]
struct Pos {
    x: usize,
    y: usize,
}
impl Pos {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn inbound(&self, min: Pos, max: Pos) -> bool {
        let x_in = min.x < self.x && self.x < max.x;
        let y_in = min.y < self.y && self.y < max.y;
        return x_in && y_in;
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Eq, PartialEq)]
struct Snake {
    pub body: VecDeque<Pos>,
    pub dir: Direction,
    pub head: Pos,
}

impl Snake {
    pub fn new(head: Pos) -> Self {
        Self {
            body: VecDeque::from([
                head,
                Pos {
                    x: head.x + 1,
                    y: head.y,
                },
                Pos {
                    x: head.x + 2,
                    y: head.y,
                },
            ]),
            dir: Direction::Left,
            head,
        }
    }

    // moves according to direction it has
    pub fn go(&mut self) {
        use Direction::*;
        match self.dir {
            Up => {
                self.body.pop_back();
                let mut new_head = self.head;
                new_head.y -= 1;
                self.body.push_front(new_head);
                self.head = new_head;
            }
            Down => {
                self.body.pop_back();
                let mut new_head = self.head;
                new_head.y += 1;
                self.body.push_front(new_head);
                self.head = new_head;
            }
            Left => {
                self.body.pop_back();
                let mut new_head = self.head;
                new_head.x -= 1;
                self.body.push_front(new_head);
                self.head = new_head;
            }

            Right => {
                self.body.pop_back();
                let mut new_head = self.head;
                new_head.x += 1;
                self.body.push_front(new_head);
                self.head = new_head;
            }
        }
    }

    pub fn grow(&mut self) {
        use Direction::*;
        let mut new_part = self.body.back().copied().unwrap();
        match self.dir {
            Up => new_part.y += 1,
            Down => new_part.y -= 1,
            Left => new_part.x += 1,
            Right => new_part.x -= 1,
        }
        self.body.push_back(new_part);
    }

    pub fn ate_itself(&self) -> bool {
        // dumb data structure choice of me
        // check if head is twice in body, because head is already in body
        return self.body.iter().filter(|&p| *p == self.head).count() >= 2;
    }

    pub fn head(&self) -> Pos {
        return self.head;
    }
    pub fn set_dir(&mut self, new_dir: Direction) {
        use Direction::*;
        match (self.dir, new_dir) {
            (Up, Down)
            | (Up, Up)
            | (Down, Up)
            | (Down, Down)
            | (Right, Left)
            | (Right, Right)
            | (Left, Right)
            | (Left, Left) => {}
            _ => self.dir = new_dir,
        }
    }
}

fn main() {
    let stdout = stdout().lock().into_raw_mode().unwrap();
    let mut game = Game::new(stdout);
    game.start();
    print!("\x1bc"); // dont know why buy restores terminal good
}
