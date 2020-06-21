import torch
from tqdm import tqdm
from alpha_zero_net import *
from mcts import *

if __name__ == '__main__':
    net = ChessNet()
    if torch.cuda.is_available():
        net.cuda()
    net.eval()

    for iter in tqdm(range(50)):
        MCTS_self_play(iter, 50, net)
