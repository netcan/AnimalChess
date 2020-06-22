import os, torch
from alpha_zero_net import ChessNet

def load_net(iter = 0):
    if not os.path.exists('model_data'): os.mkdir('model_data')
    net = ChessNet()
    if torch.cuda.is_available():
        net.cuda()

    filename = './model_data/alpha_zero_net_iter{}.pth.tar'.format(iter)
    if os.path.exists(filename):
        print('load model ', filename)
        model_data = torch.load(filename)
        net.load_state_dict(model_data)

    return net
