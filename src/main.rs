extern crate ncurses;
extern crate rand;

use ncurses::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::ops::Deref;

const FIELD_HEIGHT: i32 = 18;
const FIELD_WIDTH: i32 = 12;

struct Field {
    window: WINDOW,
}

impl Field {
    pub fn new() -> Self {
        let yoff = 1;
        let xoff = getmaxx(curscr()) / 2 - ((FIELD_WIDTH + 2) / 2);
        let window = newwin(FIELD_HEIGHT + 2, FIELD_WIDTH + 2, yoff, xoff);
        box_(window, 0, 0);
        keypad(window, true);
        intrflush(window, false);
        let mut field = Self { window };
        field.refresh();
        field
    }

    pub fn refresh(&mut self) {
        box_(self.window, 0, 0);
        wrefresh(self.window);
    }
}

impl Deref for Field {
    type Target = WINDOW;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl Drop for Field {
    fn drop(&mut self) {
        delwin(self.window);
    }
}

#[derive(Debug, Clone)]
struct Block {
    data: String,
    index: usize,
}

impl Block {
    pub fn new() -> Self {
        Self {
            data: ["....", "....", "....", "...."].concat(),
            index: 0,
        }
    }

    pub fn row(&mut self, row: &str) {
        self.replace(self.index, row);
        self.index = self.index + 1;
    }

    fn replace(&mut self, index: usize, row: &str) {
        if index >= 4 || row.len() != 4 {
            return;
        }
        self.data.replace_range((index * 4)..(index * 4 + 4), row);
    }


/*
+--+--+--+--+
| 0| 1| 2| 3| 0
+--+--+--+--+
| 4| 5| 6| 7| 1
+--+--+--+--+ 
| 8| 9|10|11| 2
+--+--+--+--+
|12|13|14|15| 3
+--+--+--+--+

+--+--+--+--+
|  |  | X|  |
+--+--+--+--+
|  | X| X|  |
+--+--+--+--+
|  | X|  |  |
+--+--+--+--+
|  |  |  |  |
+--+--+--+--+

+--+--+--+--+
|  |  |  |  |
+--+--+--+--+
|  | X| X|  |
+--+--+--+--+
|  |  | X| X|
+--+--+--+--+
|  |  |  |  |
+--+--+--+--+
*/

    pub fn rotate(&mut self) {
        let mut new = String::with_capacity(16);
        let mut y;
        let mut x;

        for (i, c) in self.data.chars().enumerate() {
            y = i / 4;
            x = i % 4;
            let idx = 12 + y - (x * 4);
            eprintln!("{}-{}", i, idx);
            new.insert(idx, c);
        }

        self.data = new;
    }

    pub fn draw(&mut self, window: WINDOW, y: i32, x: i32) {
        let mut py = y + 1;
        let mut px = x + 1;

        for v in self.data.chars().into_iter() {
            if px >= x + 5 {
                px = x + 1;
                py = py + 1;
            }
            if v != '.' {
                mvwaddch(window, py, px, v.into());
            }
            px = px + 1;
        }
    }
}

#[derive(Debug)]
struct Tetromino {
    data: Vec<Block>,
}

impl Tetromino {
    pub fn new() -> Self {
        let mut data = Vec::new();
        let mut shape;

        // I
        shape = Block::new();
        shape.row("..I.");
        shape.row("..I.");
        shape.row("..I.");
        shape.row("..I.");
        data.push(shape);

        // J
        shape = Block::new();
        shape.row("..J.");
        shape.row("..J.");
        shape.row(".JJ.");
        shape.row("....");
        data.push(shape);

        // L
        shape = Block::new();
        shape.row(".L..");
        shape.row(".L..");
        shape.row(".LL.");
        shape.row("....");
        data.push(shape);

        // O
        shape = Block::new();
        shape.row("....");
        shape.row(".OO.");
        shape.row(".OO.");
        shape.row("....");
        data.push(shape);

        // S
        shape = Block::new();
        shape.row(".S..");
        shape.row(".SS.");
        shape.row("..S.");
        shape.row("....");
        data.push(shape);

        // T
        shape = Block::new();
        shape.row("..T.");
        shape.row(".TT.");
        shape.row("..T.");
        shape.row("....");
        data.push(shape);

        // Z
        shape = Block::new();
        shape.row("..Z.");
        shape.row(".ZZ.");
        shape.row(".Z..");
        shape.row("....");
        data.push(shape);

        Self { data }
    }

    pub fn next(&self) -> Block {
        self.data.choose(&mut thread_rng()).map_or_else(
            || Block::new(),
            |b| b.clone(),
        )
    }
}

fn main() {
    let tetromino = Tetromino::new();
    let mut quit = false;
    let (mut x, mut y) = (4, 0);

    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    noecho();
    halfdelay(5);

    let mut field = Field::new();
    let (mut block, mut _next) = (tetromino.next(), tetromino.next());

    while !quit {
        // TIMING: haldelay() sleeps up to a timeout
        // INPUT
        match wgetch(*field) {
            113 => quit = true,
            KEY_UP => {
                block.rotate();
            }
            KEY_DOWN => {
                if y < FIELD_HEIGHT - 1 {
                    y = y + 1;
                }
            }
            KEY_LEFT => {
                if x > 0 {
                    x = x - 1;
                }
            }
            KEY_RIGHT => {
                if x < FIELD_WIDTH - 1 {
                    x = x + 1;
                }
            }
            _ => {}
        }

        // GAME LOGIC
        werase(*field);
        //mvwaddstr(*field, y + 1, x + 1, "X");
        block.draw(*field, y, x);

        // RENDER OUTPUT
        field.refresh();
    }

    endwin();
}
