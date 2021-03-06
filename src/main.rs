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
use std::convert::TryInto;
use std::ops::Deref;

const GAME_HEIGHT: i32 = 20;
const GAME_WIDTH: i32 = 12;
const GAME_FIELD: usize = (GAME_HEIGHT * GAME_WIDTH) as usize;

const BLOCK_WIDTH: usize = 4;
const BLOCK_SIZE: usize = BLOCK_WIDTH * BLOCK_WIDTH;

const KEY_SPACE: i32 = 32;
const KEY_QUIT: i32 = 113;
const KEY_RESTART: i32 = 114;

/// The rETRIS game.
struct Game {
    /// The window representing the main playing field of the game
    field: WINDOW,
    /// The window of the game status and help
    status: WINDOW,
    /// The state of the field
    data: [u32; GAME_FIELD],
    /// The current score
    score: i32,
    /// Game Over!
    done: bool,
    /// The level (based on max. height of rows)
    level: i32,
}

impl Game {
    /// Initialize a new game
    pub fn new() -> Self {
        let yoff = 1;
        let xoff = getmaxx(curscr()) / 2 - ((GAME_WIDTH + 2) / 2);
        let level = 10;

        let field = newwin(GAME_HEIGHT + 2, GAME_WIDTH + 2, yoff, xoff);
        let status = newwin(GAME_HEIGHT + 2, xoff - 2, yoff, 1);
        box_(field, 0, 0);

        keypad(field, true);
        intrflush(field, false);
        halfdelay(level);

        let mut game = Self {
            field,
            status,
            data: [0 as u32; GAME_FIELD],
            score: 0,
            done: false,
            level,
        };
        game.refresh();
        game
    }

    /// Update the game field
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
                row += 1;
            }
        }

        if redraw {
            self.data = data;
            wclear(**self);
        }
        for (i, ch) in self.data.iter().enumerate().filter(|(_, ch)| **ch != 0) {
            let (y, x) = Self::getyx(i);
            mvwaddch(**self, y as i32 + 1, x as i32 + 1, (*ch).into());
        }

        self.speed();

        box_(**self, 0, 0);
        wrefresh(**self);
    }

    /// Update the level and the game speed accordingly
    fn speed(&mut self) {
        let mut count = 0;

        for (i, ch) in self.data.iter().enumerate() {
            count = i;
            if *ch != 0 {
                break;
            }
        }

        // Calculate level as a percentage of the utilized rows
        let level = (((count as f32) / GAME_FIELD as f32) * 10.0) as i32;

        // Only bump the level, never lower it
        if level < self.level {
            self.level = level;
            halfdelay(self.level);
        }
    }

    /// Increase the score
    pub fn addscore(&mut self, score: i32) {
        self.score += score;
    }

    /// End the game
    pub fn gameover(&mut self) {
        self.done = true;
    }

    /// Update the game status window
    pub fn status(&mut self, block: &mut Block) {
        wclear(self.status);
        mvwaddstr(self.status, 0, 0, "rETRIS");
        mvwaddstr(self.status, 1, 0, "(reyk's TETRIS)");
        mvwaddstr(self.status, 3, 0, "Next block:");
        block.setyx(BLOCK_WIDTH as i32, BLOCK_WIDTH as i32);
        block.draw(self.status);
        mvwaddstr(self.status, 9, 0, &format!("Score: {}", self.score));
        mvwaddstr(self.status, 10, 0, &format!("Level: {}", 10 - self.level));
        if self.done {
            mvwaddstr(self.status, 12, 0, "GAME OVER!");
        }
        mvwaddstr(
            self.status,
            getmaxy(self.status) - 3,
            0,
            "left / right/ down: move",
        );
        mvwaddstr(
            self.status,
            getmaxy(self.status) - 2,
            0,
            "up: rotate   space: drop",
        );
        mvwaddstr(
            self.status,
            getmaxy(self.status) - 1,
            0,
            "r: restart       q: quit",
        );
        wrefresh(self.status);
    }

    /// Put a block on the game field stack
    pub fn store(&mut self, block: Block) {
        self.addscore(10 - self.level);
        block.store(**self, &mut self.data);
    }

    /// Get coordinates by relative index
    pub fn getyx(idx: usize) -> (usize, usize) {
        (idx / GAME_WIDTH as usize, idx % GAME_WIDTH as usize)
    }

    /// Get relative index by coordinates
    pub fn index(y: i32, x: i32) -> i32 {
        if y < 1 || x < 1 || y > GAME_HEIGHT + 1 || x > GAME_WIDTH + 1 {
            return -1;
        }
        GAME_WIDTH * (y - 1) + (x - 1)
    }

    /// Does a block pixel "fit" on the specified coordinate - is it empty?
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
        &self.field
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        delwin(self.field);
        delwin(self.status);
    }
}

/// A tetromino block
#[derive(Debug, Clone)]
struct Block {
    /// The 4x4 tetromino block
    data: [u8; BLOCK_SIZE],
    /// The type of the tetromino block
    index: usize,
    /// The current y location
    y: i32,
    /// The current x location
    x: i32,
    /// The individual id of the tetromino block
    id: i16,
}

impl Block {
    /// Return a new "empty" block
    pub fn new() -> Self {
        Self {
            data: b"................".to_owned(),
            index: 0,
            x: 0,
            y: 0,
            id: 0,
        }
    }

    /// Set the individual block id
    pub fn setid(&mut self, id: i16) {
        self.id = id;
    }

    /// Set the next row of the block to turn it into a tetromino
    pub fn row(&mut self, row: &str) {
        let i = self.index;
        if i >= BLOCK_WIDTH || row.len() != BLOCK_WIDTH {
            return;
        }
        self.data[(i * BLOCK_WIDTH)..(i * BLOCK_WIDTH + BLOCK_WIDTH)]
            .copy_from_slice(row.as_bytes());
        self.index = i + 1;
    }

    /// Store the coordinates of the block
    pub fn setyx(&mut self, y: i32, x: i32) {
        self.y = y;
        self.x = x;
    }

    /// Get the coordinates of the block
    pub fn getyx(idx: usize) -> (usize, usize) {
        (idx / BLOCK_WIDTH, idx % BLOCK_WIDTH)
    }

    /// Rotate the block on the game field
    pub fn rotate(&mut self, game: &Game) {
        let mut new: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];

        // clear block
        self.clear(**game);

        // rotate each pixel by 90 degrees cw
        for (i, c) in self.data.iter().enumerate() {
            let (y, x) = Self::getyx(i);
            let idx = BLOCK_WIDTH * (BLOCK_WIDTH - 1) + y - x * BLOCK_WIDTH;
            // for ccw:
            //let idx = BLOCK_WIDTH - 1 - y + x * BLOCK_WIDTH;
            new[idx] = *c;
        }

        let old = self.data;
        self.data = new;

        if !self.fits(&game, self.y, self.x) {
            // revert to previous
            self.data = old;
            return;
        }
    }

    /// Draw the block on the specified window
    pub fn draw(&self, window: WINDOW) {
        self.fill(window, false, &mut []);
    }

    /// Clear the block from the specified window
    pub fn clear(&self, window: WINDOW) {
        self.fill(window, true, &mut []);
    }

    /// Draw the block on the specified window and save its pixels
    pub fn store(&self, window: WINDOW, data: &mut [u32]) {
        self.fill(window, false, data);
    }

    fn fill(&self, window: WINDOW, clear: bool, data: &mut [u32]) {
        let mut py = self.y;
        let mut px = self.x;

        for v in self.data.iter() {
            let c = *v as char;
            if px >= self.x + BLOCK_WIDTH as i32 {
                px = self.x;
                py += 1;
            }
            if py > 0 && c != '.' {
                let mut ch: u32 = c.into();
                if clear {
                    ch = ' '.into();
                } else if has_colors() {
                    ch = (ACS_BLOCK() | COLOR_PAIR(self.id)).try_into().unwrap();
                }
                mvwaddch(window, py, px, ch.into());

                let idx = Game::index(py, px);
                if idx > 0 && data.len() >= idx as usize {
                    data[idx as usize] = ch;
                }
            }
            px += 1;
        }
    }

    /// Does the block fit on the game field?
    pub fn fits(&self, game: &Game, y: i32, x: i32) -> bool {
        let mut py = y;
        let mut px = x;

        for v in self.data.iter() {
            let c = *v as char;
            if px >= x + BLOCK_WIDTH as i32 {
                px = x;
                py += 1;
            }
            if c != '.'
                && (px < 1 || px > GAME_WIDTH || py > GAME_HEIGHT || (py > 0 && !game.fits(py, px)))
            {
                return false;
            }
            px += 1;
        }
        true
    }
}

/// All tetromino blocks
#[derive(Debug)]
struct Tetromino {
    /// A vector of all tetrominos (I, J, L, O, S, T, Z)
    data: Vec<Block>,
}

impl Tetromino {
    /// Create the tetrominos
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
        self.data
            .choose(&mut thread_rng())
            .map_or_else(Block::new, |b| b.clone())
    }
}

/// Start a new game
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
                    y += 1;
                }
            }
            KEY_LEFT => {
                if block.fits(&game, y, x - 1) {
                    x -= 1;
                }
            }
            KEY_RIGHT => {
                if block.fits(&game, y, x + 1) {
                    x += 1;
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
            y += 1;
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

/// rETRIS!
fn main() {
    let tetromino = Tetromino::new();

    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    noecho();

    if has_colors() {
        start_color();

        // Set the block colors by index
        init_pair(1, COLOR_BLACK, COLOR_CYAN);
        init_pair(2, COLOR_BLACK, COLOR_BLUE);
        init_pair(3, COLOR_BLACK, COLOR_WHITE);
        init_pair(4, COLOR_BLACK, COLOR_YELLOW);
        init_pair(5, COLOR_BLACK, COLOR_GREEN);
        init_pair(6, COLOR_BLACK, COLOR_MAGENTA);
        init_pair(7, COLOR_BLACK, COLOR_RED);
    }

    engine(tetromino);

    endwin();
}
