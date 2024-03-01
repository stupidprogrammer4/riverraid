use std::{collections::VecDeque, io::{stdout, Stdout, Write}, thread, time::Duration};
use rand::prelude::*;

use crossterm::{
    cursor::{Hide, MoveTo, Show}, event::{poll, read, Event, KeyCode}, style::{Color, Print, SetBackgroundColor}, terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType}, ExecutableCommand, QueueableCommand
};

struct Game {
    maxr: u16,
    maxc: u16,
    player_c: u16,
    river: VecDeque<(u16, u16)>,
    foods: VecDeque<(bool, u16)>,
    bombs: VecDeque<(bool, u16)>,
    score: i32,
    alive: bool,
    next_en: u16,
    next_st: u16
}

fn handler(game: &mut Game) -> std::io::Result<()> {
    if game.player_c <= game.river[(game.maxr-1) as usize].0 || game.player_c >= game.river[(game.maxr-1) as usize].1 {
        game.alive = false;
        return Ok(())
    }
    if game.bombs[(game.maxr-1) as usize].0 && game.player_c == game.bombs[(game.maxr-1) as usize].1 {
        game.score -= 1;
        if game.score < 0 {
            game.alive = false;
            return Ok(())
        }
    }
    if game.foods[(game.maxr-1) as usize].0 && game.player_c == game.foods[(game.maxr-1) as usize].1 {
        game.score += 1
    }
    let mut rng = thread_rng();
    game.river.pop_back();
    game.foods.pop_back();
    game.bombs.pop_back();
    let mut st: u16 = game.river[0].0;
    let mut en: u16 = game.river[0].1; 
    if game.next_en < game.river[0].1 {
        en -= 1
    }
    if game.next_en > game.river[0].1 {
        en += 1
    }
    if game.next_st < game.river[0].0 {
        st -= 1
    }
    if game.next_st > game.river[0].0 {
        st += 1
    }
    game.river.push_front((st, en));
    let food_pos: f64 = rng.gen();
    let bomb_pos: f64 = rng.gen();
    if bomb_pos < 0.05 {
        game.bombs.push_front((true, rng.gen_range(st+1..en-1)));
    }
    else {
        game.bombs.push_front((false, 0))
    }
    if food_pos < 0.06 {
        game.foods.push_front((true, rng.gen_range(st+1..en-1)))
    }
    else {
        game.foods.push_front((false, 0))
    }

    if game.foods[0].1 == game.bombs[0].1 {
        game.foods[0].0 = false
    }
    

    if game.river[0].1 == game.next_en && game.river[0].0 == game.next_st {
        if rng.gen_range(0..10) > 7 {
            let mut next_st = rng.gen_range((game.river[0].0 as i32) - 30..(game.river[0].1 as i32) + 5);
            if next_st <= 0 {
                next_st += 30;
                game.next_st = next_st as u16;
            } else {
                game.next_st = next_st as u16;
            }
            game.next_en = rng.gen_range(game.next_st+12..game.next_st+25);
            if game.next_en >= game.maxc-1 {
                game.next_en = game.maxc-1;
                game.next_st = game.maxc-1-rng.gen_range(12..25)
            }
        }
    }
    
    return Ok(())
}

fn draw(game: &Game, mut sc: &Stdout) -> std::io::Result<()> {
    sc.queue(Clear(ClearType::All))?;
    for i in 0..game.river.len() {
        sc.queue(SetBackgroundColor(Color::Green))?
          .queue(MoveTo(2, 2))?
          .queue(Print(format!("Score: {}", game.score)))?
          .queue(MoveTo(0, i as u16))?
          .queue(Print(" ".repeat(game.river[i].0 as usize)))?
          .queue(MoveTo(game.river[i].1, i as u16))?
          .queue(Print(" ".repeat((game.maxc - game.river[i].1) as usize)))?
          .queue(MoveTo(game.river[i].0+1, i as u16))?
          .queue(SetBackgroundColor(Color::Blue))?
          .queue(Print(" ".repeat((game.river[i].1 - game.river[i].0 - 1) as usize)))?;
        if game.foods[i].0 {
            sc.queue(MoveTo(game.foods[i].1, i as u16))?
              .queue(Print("⛽"))?;
        } 
        if game.bombs[i].0 {
            sc.queue(MoveTo(game.bombs[i].1, i as u16))?
              .queue(Print("💣"))?;
        }
    }
    sc.queue(MoveTo(game.player_c, game.maxr-1))?
      .queue(Print("⛵"))?;

    sc.flush()?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut sc = stdout();
    enable_raw_mode()?;
    let (maxc, maxr) = size()?;
    sc.execute(Hide)?;


    let mut game = Game {
        maxc: maxc,
        maxr: maxr,
        score: 0,
        foods: VecDeque::from(vec![(false, 0); maxr as usize]),
        bombs: VecDeque::from(vec![(false, 0); maxr as usize]),
        player_c: maxc/2,
        river: VecDeque::from(vec![(maxc/2-5, maxc/2+5); maxr as usize]),
        alive: true,
        next_en: maxc/2+10,
        next_st: maxc/2-10
    };
    while game.alive {
        while poll(Duration::from_millis(10))? {
            let key = read().unwrap();
            match key {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char('q') => {game.alive = false},
                        KeyCode::Char('a') => { if game.player_c > 0 {game.player_c -= 1;} },
                        KeyCode::Char('d') => { if game.player_c < maxc-1 {game.player_c += 1;} }
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        handler(&mut game)?;
        draw(&game, &sc)?;   
        thread::sleep(Duration::from_millis(200))  
    }
    sc.execute(Clear(ClearType::All))?
      .execute(MoveTo(maxc/2, maxr/2))?
      .execute(Print("Thanks for playing!"))?;
    thread::sleep(Duration::from_millis(1000));
    disable_raw_mode()?;
    sc.execute(Show)?;
    Ok(())
}
