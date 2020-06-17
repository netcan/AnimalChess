## Animal Chess
Animal Fight Chess Game（斗兽棋） written in rust.
![assets/animal_chess.png](assets/animal_chess.png)

## How To Play
To win the game, one player must successfully move any animal into the Den（兽穴） of the opponent.
See rules:
- [http://ancientchess.com/page/play-doushouqi.htm](http://ancientchess.com/page/play-doushouqi.htm)
- [https://en.wikipedia.org/wiki/Jungle_(board_game)](https://en.wikipedia.org/wiki/Jungle_(board_game)).

## How To Run
It need rust nightly version.
```
$ rustup default nightly
```

```
$ git clone https://github.com/netcan/AnimalChess.git
$ cd AnimalChess
$ cargo run --release
```

## Todo
- [X] Seperate `game.rs` to `gui.rs` and `board.rs`
- [x] Add `Monte Carlo Tree Search` Algorithm
- [X] benmark board operation
- [ ] export module for python3
- [ ] encode/decode move
- [ ] generate fen
- [ ] patch sdl2 to support `load_texture` from buffer
