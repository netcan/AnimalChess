import torch
from tqdm import tqdm
from mcts import MCTS_self_play
from train import train

if __name__ == '__main__':
    for iter in tqdm(range(0, 100)):
        # mtcs serach
        MCTS_self_play(iter, 30, 1)

        # training
        train(iter)
