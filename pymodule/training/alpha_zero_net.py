import torch
import torch.nn as nn
import torch.nn.functional as F

class ConvBlock(nn.Module):
    def __init__(self):
        super(ConvBlock, self).__init__()
        self.conv = nn.Conv2d(16, 256, 3, stride=1, padding=1)
        self.bn = nn.BatchNorm2d(256)

    def forward(self, s):
        s = s.view(-1, 16, 9, 7) # batch_size x channels x board_x x board_y
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
