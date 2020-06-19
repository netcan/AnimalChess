from random import choice
from animal_chess_pymodule import Board

def gen_and_move_chess(max_times: int = 500):
    board = Board()
    for _ in range(max_times):
        all_step = board.generate_all_steps()
        if all_step: board.move_chess(all_step[0])
        else: break

def self_play():
    board = Board()
    while True:
        all_step = board.generate_all_steps()
        if all_step: board.move_chess(choice(all_step))
        else: break


