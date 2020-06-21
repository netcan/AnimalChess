import numpy as np
from animal_chess_pymodule import *
from alpha_zero_net import ChessNet

class Node:
    def __init__(game, move, parent = None):
        self.is_expanded = False
        self.parent = parent
        self.game = game
        self.move = move
        self.children = {}
        self.child_priors = np.zeros([252], dtype=np.float)
        self.child_total_value = np.zeros([252], dtype=np.float)
        self.child_number_visits = np.zeros([252], dtype=np.float)
        self.action_idxes = []

    @property
    def number_visits(self):
        return self.parent.child_number_visits[self.move]

    @number_visits.setter
    def number_visits(self, value):
        self.parent.child_number_visits[self.move] = value

    @property
    def total_value(self):
        return self.parent.child_total_value[self.move]

    @total_value.setter
    def total_value(self, value):
        self.parent.child_total_value[self.move] = value

    def child_Q(self):
        return self.child_total_value / (self.child_number_visits + 1)

    def child_U(self):
        return abs(self.child_priors) * (math.sqrt(self.number_visits)
                                         / (self.child_number_visits + 1))

    def best_child(self):
        if self.action_idxes != []:
            bestmove = self.child_Q() + self.child_U()
            bestmove = np.argmax(bestmove[self.action_idxes])
        else:
            bestmove = np.argmax(self.child_Q() + self.child_U())
        return bestmove

    def maybe_add_child(self, move):
        if move not in self.children:
            self.children[move] = Node(self.game, move, parent=self)
        return self.children[move]

    def select_leaf(self):
        current = self
        while current.is_expanded:
            bestmove = current.best_child()
            current.board.move_chess(best_move)
            current = current.maybe_add_child(best_move)
        return current

    def add_dirichlet_noise(self, action_idxs, child_priors):
        valid_child_priors = child_priors[action_idxs] # select only legal moves entries in child_priors array
        valid_child_priors = 0.75*valid_child_priors + 0.25*np.random.dirichlet(np.zeros([len(valid_child_priors)], dtype=np.float32)+0.3)
        child_priors[action_idxs] = valid_child_priors
        return child_priors

    def expand(self, child_priors):
        self.is_expanded = True
        self.action_idxs = self.game.generate_all_steps()
        if self.action_idxs == []:
            self.is_expanded = False
        self.child_priors = child_priors

        mask = np.ones(len(self.child_priors), np.bool)
        mask[self.action_idxs] = False
        self.child_priors[mask] = 0.0

    def backup(self, value_estimate: float):
        current = self
        while current.parent is not None:
            current.number_visits += 1
            if current.game.role() == Role.BLACK: # same as current.parent.game.player = 0
                current.total_value += (1*value_estimate) # value estimate +1 = white win
            elif current.game.role() == Role.RED: # same as current.parent.game.player = 1
                current.total_value += (-1*value_estimate)

            current.game.undo_move()
            current = current.parent

class DummyNode(object):
    def __init__(self):
        self.parent = None
        self.child_total_value = collections.defaultdict(float)
        self.child_number_visits = collections.defaultdict(float)

def UCT_search(game_state, times, net):
    root = Node(game_state, move=None, parent=DummyNode())
    for i in range(times):
        leaf = root.select_leaf()
        encoded_s = ed.encode_board(leaf.game); encoded_s = encoded_s.transpose(2,0,1)
        encoded_s = torch.from_numpy(encoded_s).float().cuda()
        child_priors, value_estimate = net(encoded_s)
        child_priors = child_priors.detach().cpu().numpy().reshape(-1); value_estimate = value_estimate.item()
        if leaf.game.check_status() == True and leaf.game.in_check_possible_moves() == []: # if checkmate
            leaf.backup(value_estimate); continue
        leaf.expand(child_priors) # need to make sure valid moves
        leaf.backup(value_estimate)
    return np.argmax(root.child_number_visits), root
