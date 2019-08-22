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
    blocks: Vec<Block>,
    data: [char; (FIELD_HEIGHT * FIELD_WIDTH) as usize],
}

impl Field {
    pub fn new() -> Self {
        let yoff = 1;
        let xoff = getmaxx(curscr()) / 2 - ((FIELD_WIDTH + 2) / 2);
        let window = newwin(FIELD_HEIGHT + 2, FIELD_WIDTH + 2, yoff, xoff);
        box_(window, 0, 0);
        keypad(window, true);
        intrflush(window, false);
        let mut field = Self {
            window,
            blocks: Vec::new(),
            data: [0 as char; (FIELD_HEIGHT * FIELD_WIDTH) as usize],
        };
        field.refresh();
        field
    }

    pub fn refresh(&mut self) {
        let mut data = self.data.clone();
        box_(**self, 0, 0);
        for block in self.blocks.iter() {
            block.store(&self, &mut data);
        }
        wrefresh(**self);
        self.data = data;
    }

    pub fn store(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn index(y: i32, x: i32) -> i32 {
        if y < 1 || x < 1 || y > FIELD_HEIGHT + 1 || x > FIELD_WIDTH + 1 {
            return -1;
        }
        (y - 1) * FIELD_WIDTH + (x - 1)
    }

    pub fn fits(&self, y: i32, x: i32) -> bool {
        let idx = Self::index(y, x);
        if idx < 0 || self.data[idx as usize] != 0.into() {
            return false;
        }
        true
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
    data: [u8; 16],
    index: usize,
    y: i32,
    x: i32,
}

impl Block {
    pub fn new() -> Self {
        Self {
            data: b"................".to_owned(),
            index: 0,
            x: 0,
            y: 0,
        }
    }

    pub fn row(&mut self, row: &str) {
        let i = self.index;
        if i >= 4 || row.len() != 4 {
            return;
        }
        self.data[(i * 4)..(i * 4 + 4)].copy_from_slice(row.as_bytes());
        self.index = i + 1;
    }

    pub fn setyx(&mut self, y: i32, x: i32) {
        self.y = y;
        self.x = x;
    }

    pub fn getyx(idx: usize) -> (usize, usize) {
        (idx / 4, idx % 4)
    }

    pub fn rotate(&mut self, field: &Field) {
        let mut new: [u8; 16] = [0; 16];

        self.clear(&field);

        for (i, c) in self.data.into_iter().enumerate() {
            let (y, x) = Self::getyx(i);
            let idx = 12 + y - (x * 4);
            new[idx] = *c;
        }

        self.data = new;
    }

    pub fn draw(&self, field: &Field) {
        self.fill(&field, false, &mut []);
    }

    pub fn clear(&self, field: &Field) {
        self.fill(&field, true, &mut []);
    }

    pub fn store(&self, field: &Field, data: &mut [char]) {
        self.fill(&field, false, data);
    }

    fn fill(&self, field: &Field, clear: bool, data: &mut [char]) {
        let mut py = self.y;
        let mut px = self.x;

        for v in self.data.into_iter() {
            let mut c = *v as char;
            if px >= self.x + 4 {
                px = self.x;
                py = py + 1;
            }
            if c != '.' {
                if clear {
                    c = ' ';
                }
                mvwaddch(**field, py, px, c.into());

                let idx = Field::index(py, px);
                if idx > 0 && data.len() >= idx as usize {
                    data[idx as usize] = c;
                }
            }
            px = px + 1;
        }
    }

    pub fn fits(&mut self, field: &Field, y: i32, x: i32) -> bool {
        let mut py = y;
        let mut px = x;

        for v in self.data.into_iter() {
            let c = *v as char;
            if px >= x + 4 {
                px = x;
                py = py + 1;
            }
            if c != '.' {
                if px < 1 || px > FIELD_WIDTH || py > FIELD_HEIGHT ||
                    (py > 0 && !field.fits(py, px))
                {
                    return false;
                }
            }
            px = px + 1;
        }
        true
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
    let (mut x, mut y) = (5, -3);

    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    noecho();
    halfdelay(5);

    let mut field = Field::new();
    let (mut block, mut next) = (tetromino.next(), tetromino.next());

    while !quit {
        // TIMING: haldelay() sleeps up to a timeout
        // INPUT
        match wgetch(*field) {
            113 => quit = true,
            110 => {
                block = next;
                next = tetromino.next();
            }
            KEY_UP => {
                block.rotate(&field);
            }
            KEY_DOWN => {
                if block.fits(&field, y + 1, x) {
                    y = y + 1;
                }
            }
            KEY_LEFT => {
                if block.fits(&field, y, x - 1) {
                    x = x - 1;
                }
            }
            KEY_RIGHT => {
                if block.fits(&field, y, x + 1) {
                    x = x + 1;
                }
            }
            _ => {}
        }

        // GAME LOGIC
        block.clear(&field);
        block.setyx(y, x);
        block.draw(&field);
        if !block.fits(&field, y + 1, x) {
            field.store(block);
            block = next;
            next = tetromino.next();
            x = 5;
            y = -3;
        } else {
            y = y + 1;
        }

        // RENDER OUTPUT
        field.refresh();
    }

    endwin();
}
