//
// Copyright (c) 2019 Reyk Floeter <contact@reykfloeter.com>
//
// Permission to use, copy, modify, and distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//

extern crate ncurses;
extern crate rand;

use ncurses::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::ops::Deref;

const GAME_HEIGHT: i32 = 18;
const GAME_WIDTH: i32 = 12;
const GAME_FIELD: usize = (GAME_HEIGHT * GAME_WIDTH) as usize;

const KEY_SPACE: i32 = 32;
const KEY_QUIT: i32 = 113;
const KEY_RESTART: i32 = 114;

/// The rETRIS game.
struct Game {
    window: WINDOW,
    status: WINDOW,
    data: [u32; GAME_FIELD],
    score: i32,
    done: bool,
    level: i32,
}

impl Game {
    pub fn new() -> Self {
        let yoff = 1;
        let xoff = getmaxx(curscr()) / 2 - ((GAME_WIDTH + 2) / 2);
        let level = 10;

        let window = newwin(GAME_HEIGHT + 2, GAME_WIDTH + 2, yoff, xoff);
        let status = newwin(GAME_HEIGHT + 2, xoff - 2, yoff, 1);
        box_(window, 0, 0);

        keypad(window, true);
        intrflush(window, false);
        halfdelay(level);

        let mut game = Self {
            window,
            status,
            data: [0 as u32; GAME_FIELD],
            score: 0,
            done: false,
            level,
        };
        game.refresh();
        game
    }

    pub fn refresh(&mut self) {
        let mut data: [u32; GAME_FIELD] = [0 as u32; GAME_FIELD];
        let mut redraw = false;
        let mut row = 1;

        // Remove full rows
        for r in self.data.chunks(GAME_WIDTH as usize).rev() {
            if !r.contains(&0) {
                redraw = true;
            } else {
                let j = GAME_FIELD - (row * GAME_WIDTH as usize);
                data[j..(j + GAME_WIDTH as usize)].copy_from_slice(r);
                row = row + 1;
            }
        }

        if redraw {
            self.data = data;
            wclear(**self);
        }
        for (i, ch) in self.data.into_iter().enumerate().filter(
            |(_, ch)| **ch != 0,
        )
        {
            let (y, x) = Self::getyx(i);
            mvwaddch(**self, y as i32 + 1, x as i32 + 1, *ch);
        }

        self.speed();

        box_(**self, 0, 0);
        wrefresh(**self);
    }

    fn speed(&mut self) {
        let mut count = 0;

        for (i, ch) in self.data.into_iter().enumerate() {
            count = i;
            if *ch != 0 {
                break;
            }
        }

        // Calculate level as a percentage of the utilized rows
        let level = (((count as f32) / GAME_FIELD as f32).abs() * 10 as f32) as i32;

        // Only bump the level, never lower it
        if level < self.level {
            self.level = level;
            halfdelay(self.level);
        }
    }

    pub fn addscore(&mut self, score: i32) {
        self.score = self.score + score;
    }

    pub fn gameover(&mut self) {
        self.done = true;
    }

    pub fn status(&mut self, block: &mut Block) {
        wclear(self.status);
        mvwaddstr(self.status, 0, 0, "rETRIS");
        mvwaddstr(self.status, 1, 0, "(reyk's TETRIS)");
        mvwaddstr(self.status, 3, 0, "Next block:");
        block.setyx(4, 4);
        block.draw(self.status);
        mvwaddstr(self.status, 9, 0, &format!("Score: {}", self.score));
        mvwaddstr(self.status, 10, 0, &format!("Level: {}", 10 - self.level));
        if self.done {
            mvwaddstr(self.status, 12, 0, "GAME OVER!");
        }
        mvwaddstr(
            self.status,
            getmaxy(self.status) - 1,
            0,
            "q: quit   r: restart",
        );
        wrefresh(self.status);
    }

    pub fn store(&mut self, block: Block) {
        self.addscore(10 - self.level);
        block.store(**self, &mut self.data);
    }

    pub fn getyx(idx: usize) -> (usize, usize) {
        (idx / GAME_WIDTH as usize, idx % GAME_WIDTH as usize)
    }

    pub fn index(y: i32, x: i32) -> i32 {
        if y < 1 || x < 1 || y > GAME_HEIGHT + 1 || x > GAME_WIDTH + 1 {
            return -1;
        }
        (y - 1) * GAME_WIDTH + (x - 1)
    }

    pub fn fits(&self, y: i32, x: i32) -> bool {
        let idx = Self::index(y, x);
        if idx < 0 || self.data[idx as usize] != 0 {
            return false;
        }
        true
    }
}

impl Deref for Game {
    type Target = WINDOW;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        delwin(self.window);
    }
}

/// A tetromino block
#[derive(Debug, Clone)]
struct Block {
    data: [u8; 16],
    index: usize,
    y: i32,
    x: i32,
    id: i16,
}

impl Block {
    pub fn new() -> Self {
        Self {
            data: b"................".to_owned(),
            index: 0,
            x: 0,
            y: 0,
            id: 0,
        }
    }

    pub fn setid(&mut self, id: i16) {
        self.id = id;
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

    pub fn rotate(&mut self, game: &Game) {
        let mut new: [u8; 16] = [0; 16];

        // clear block
        self.clear(**game);

        for (i, c) in self.data.into_iter().enumerate() {
            let (y, x) = Self::getyx(i);
            let idx = 12 + y - (x * 4);
            new[idx] = *c;
        }

        let old = self.data;
        self.data = new;

        if !self.fits(&game, self.y, self.x) {
            self.data = old;
            return;
        }
    }

    pub fn draw(&self, window: WINDOW) {
        self.fill(window, false, &mut []);
    }

    pub fn clear(&self, window: WINDOW) {
        self.fill(window, true, &mut []);
    }

    pub fn store(&self, window: WINDOW, data: &mut [u32]) {
        self.fill(window, false, data);
    }

    fn fill(&self, window: WINDOW, clear: bool, data: &mut [u32]) {
        let mut py = self.y;
        let mut px = self.x;

        for v in self.data.into_iter() {
            let c = *v as char;
            if px >= self.x + 4 {
                px = self.x;
                py = py + 1;
            }
            if py > 0 && c != '.' {
                let mut ch = c.into();
                if clear {
                    ch = ' '.into();
                } else if has_colors() {
                    ch = ACS_BLOCK() | COLOR_PAIR(self.id);
                }
                mvwaddch(window, py, px, ch);

                let idx = Game::index(py, px);
                if idx > 0 && data.len() >= idx as usize {
                    data[idx as usize] = ch;
                }
            }
            px = px + 1;
        }
    }

    pub fn fits(&self, game: &Game, y: i32, x: i32) -> bool {
        let mut py = y;
        let mut px = x;

        for v in self.data.into_iter() {
            let c = *v as char;
            if px >= x + 4 {
                px = x;
                py = py + 1;
            }
            if c != '.' {
                if px < 1 || px > GAME_WIDTH || py > GAME_HEIGHT || (py > 0 && !game.fits(py, px)) {
                    return false;
                }
            }
            px = px + 1;
        }
        true
    }
}

/// All tetromino blocks
#[derive(Debug)]
struct Tetromino {
    data: Vec<Block>,
}

impl Tetromino {
    pub fn new() -> Self {
        let mut data = Vec::new();
        let mut block;

        // I
        block = Block::new();
        block.setid(1);
        block.row("..I.");
        block.row("..I.");
        block.row("..I.");
        block.row("..I.");
        data.push(block);

        // J
        block = Block::new();
        block.setid(2);
        block.row("..J.");
        block.row("..J.");
        block.row(".JJ.");
        block.row("....");
        data.push(block);

        // L
        block = Block::new();
        block.setid(3);
        block.row(".L..");
        block.row(".L..");
        block.row(".LL.");
        block.row("....");
        data.push(block);

        // O
        block = Block::new();
        block.setid(4);
        block.row("....");
        block.row(".OO.");
        block.row(".OO.");
        block.row("....");
        data.push(block);

        // S
        block = Block::new();
        block.setid(5);
        block.row(".S..");
        block.row(".SS.");
        block.row("..S.");
        block.row("....");
        data.push(block);

        // T
        block = Block::new();
        block.setid(6);
        block.row("..T.");
        block.row(".TT.");
        block.row("..T.");
        block.row("....");
        data.push(block);

        // Z
        block = Block::new();
        block.setid(7);
        block.row("..Z.");
        block.row(".ZZ.");
        block.row(".Z..");
        block.row("....");
        data.push(block);

        Self { data }
    }

    pub fn next(&self) -> Block {
        self.data.choose(&mut thread_rng()).map_or_else(
            || Block::new(),
            |b| b.clone(),
        )
    }
}

fn engine(tetromino: Tetromino) {
    let mut quit = false;
    let (mut x, mut y) = (5, -1);
    let mut game = Game::new();
    let (mut block, mut next) = (tetromino.next(), tetromino.next());
    game.status(&mut next);

    while !quit {
        // Handle input
        match wgetch(*game) {
            KEY_QUIT => quit = true,
            KEY_RESTART => return engine(tetromino),
            KEY_SPACE => {
                // Jump to last possible line
                for py in (y..getmaxy(*game)).rev() {
                    if block.fits(&game, py, x) {
                        game.addscore(py - y);
                        y = py;
                        break;
                    }
                }
            }
            KEY_UP => {
                block.rotate(&game);
            }
            KEY_DOWN => {
                if block.fits(&game, y + 1, x) {
                    y = y + 1;
                }
            }
            KEY_LEFT => {
                if block.fits(&game, y, x - 1) {
                    x = x - 1;
                }
            }
            KEY_RIGHT => {
                if block.fits(&game, y, x + 1) {
                    x = x + 1;
                }
            }
            _ => {}
        }

        // Core logic
        block.clear(*game);
        block.setyx(y, x);
        block.draw(*game);

        // Store block and create a new one if the previous doesn't fit
        if !block.fits(&game, y + 1, x) {
            game.store(block);
            block = next;
            next = tetromino.next();
            beep();
            x = 5;
            y = -1;
            game.status(&mut next);
        } else {
            y = y + 1;
        }

        // End game if the new block doesn't fit
        if quit || !block.fits(&game, y, x) {
            game.gameover();
            game.status(&mut next);

            quit = false;
            while !quit {
                match wgetch(*game) {
                    KEY_QUIT => quit = true,
                    KEY_RESTART => return engine(tetromino),
                    _ => {}
                }

            }
        }

        // Render output
        game.refresh();
    }
}

fn main() {
    let tetromino = Tetromino::new();

    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    noecho();

    if has_colors() {
        start_color();
        init_pair(1, COLOR_CYAN, COLOR_BLACK);
        init_pair(2, COLOR_BLUE, COLOR_BLACK);
        init_pair(3, COLOR_WHITE, COLOR_BLACK);
        init_pair(4, COLOR_YELLOW, COLOR_BLACK);
        init_pair(5, COLOR_GREEN, COLOR_BLACK);
        init_pair(6, COLOR_MAGENTA, COLOR_BLACK);
        init_pair(7, COLOR_RED, COLOR_BLACK);
    }

    engine(tetromino);

    endwin();
}
