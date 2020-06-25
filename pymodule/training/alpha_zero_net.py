import torch, os, pickle
import torch.nn as nn
import torch.nn.functional as F

class ConvBlock(nn.Module):
    def __init__(self):
        super(ConvBlock, self).__init__()
        self.conv = nn.Conv2d(18, 256, 3, stride=1, padding=1)
        self.bn = nn.BatchNorm2d(256)

    def forward(self, s):
        s = s.view(-1, 18, 9, 7) # batch_size x channels x board_x x board_y
        return F.relu(self.bn(self.conv(s)))

class ResBlock(nn.Module):
    def __init__(self, inplanes=256, planes=256):
        super(ResBlock, self).__init__()
        self.conv1 = nn.Conv2d(inplanes, planes, 3, stride=1, padding=1, bias=False)
        self.bn1 = nn.BatchNorm2d(planes)
        self.conv2 = nn.Conv2d(planes, planes, 3, stride=1, padding=1, bias=False)
        self.bn2 = nn.BatchNorm2d(planes)

    def forward(self, x):
        residual = x
        out = self.conv1(x)
        out = self.bn1(out)
        out = F.relu(out)
        out = self.conv2(out)
        out = self.bn2(out)
        out += residual
        out = F.relu(out)
        return out

class OutBlock(nn.Module):
    def __init__(self):
        super(OutBlock, self).__init__()
        self.conv1 = nn.Conv2d(256, 1, 1) # value
        self.bn1 = nn.BatchNorm2d(1)
        self.fc1 = nn.Linear(7 * 9, 63)
        self.fc2 = nn.Linear(63, 1)

        self.conv2 = nn.Conv2d(256, 128, 1) # policy
        self.bn2 = nn.BatchNorm2d(128)
        self.softmax = nn.Softmax(1)
        self.fc = nn.Linear(128 * 9 * 7, 4 * 9 * 7)

    def forward(self, s):
        v = F.relu(self.bn1(self.conv1(s)))
        v = v.view(-1, 9 * 7)
        v = F.relu(self.fc1(v))
        v = torch.tanh(self.fc2(v))

        p = F.relu(self.bn2(self.conv2(s)))
        p = p.view(-1, 128 * 9 * 7)
        p = self.fc(p)
        p = self.softmax(p)
        return p, v

class ChessNet(nn.Module):
    def __init__(self):
        super(ChessNet, self).__init__()
        self.conv = ConvBlock()
        for block in range(19):
            setattr(self, "res_%i" % block, ResBlock())
        self.out = OutBlock()

    def forward(self, s):
        s = self.conv(s)
        for block in range(19):
            s = getattr(self, "res_%i" % block)(s)
        return self.out(s)

class AlphaLoss(nn.Module):
    def __init__(self):
        super(AlphaLoss, self).__init__()

    def forward(self, y_value, v, y_policy, policy):
        value_error = (y_value - v) ** 2
        policy_error = torch.sum((-policy*
                                (1e-8 + y_policy.float()).float().log()), 1)
        total_error = (value_error.view(-1).float() + policy_error).mean()
        return total_error

