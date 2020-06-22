import torch, math, os
import numpy as np
import pickle, collections, time
from tqdm import tqdm
from animal_chess_pymodule import *
from alpha_zero_net import ChessNet

class Node():
    MAX_ACTION = 252
    def __init__(self, game, move, parent = None):
        self.is_expanded = False
        self.parent = parent
        self.game = game
        self.move = move
        self.children = {}
        self.child_priors = np.zeros([Node.MAX_ACTION], dtype=np.float)
        self.child_total_value = np.zeros([Node.MAX_ACTION], dtype=np.float)
        self.child_number_visits = np.zeros([Node.MAX_ACTION], dtype=np.float)
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
            bestmove = self.action_idxes[np.argmax(bestmove[self.action_idxes])]
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
            best_move = current.best_child()
            current.game.move_chess(best_move)
            current = current.maybe_add_child(best_move)
        return current

    def add_dirichlet_noise(self):
        valid_child_priors = self.child_priors[self.action_idxes] # select only legal moves entries in child_priors array
        valid_child_priors = 0.75*valid_child_priors + 0.25*np.random.dirichlet(np.zeros([len(valid_child_priors)], dtype=np.float32)+0.3)
        self.child_priors[self.action_idxes] = valid_child_priors

    def expand(self, child_priors):
        self.is_expanded = True
        self.action_idxes = self.game.generate_all_steps()
        if self.action_idxes == []:
            self.is_expanded = False
        self.child_priors = child_priors

        mask = np.ones(len(self.child_priors), np.bool)
        mask[self.action_idxes] = False
        self.child_priors[mask] = 0.0

        if isinstance(self.parent, DummyNode): # root
            self.add_dirichlet_noise()

    def backup(self, value_estimate: float):
        current = self
        while current.parent is not None:
            current.number_visits += 1
            if current.game.role() == Role.BLACK: # same as current.parent.game.player = 0
                current.total_value += (1*value_estimate) # value estimate +1 = white win
            elif current.game.role() == Role.RED: # same as current.parent.game.player = 1
                current.total_value += (-1*value_estimate)

            if not isinstance(current.parent, DummyNode): # root
                current.game.undo_move()
            current = current.parent

    def get_policy(self):
        policy = np.zeros(Node.MAX_ACTION)
        for idx in np.where(self.child_number_visits!=0)[0]:
            policy[idx] = self.child_number_visits[idx]/self.child_number_visits.sum()
        return policy

class DummyNode(object):
    def __init__(self):
        self.parent = None
        self.child_total_value = collections.defaultdict(float)
        self.child_number_visits = collections.defaultdict(float)

def UCT_search(game_state, times, net):
    root = Node(game_state, move=None, parent=DummyNode())
    for i in (range(times)):
        leaf = root.select_leaf()
        encoded_s = torch.from_numpy(np.array(leaf.game.encode_board())).float()
        if torch.cuda.is_available(): encoded_s = encoded_s.cuda()

        child_priors, value_estimate = net(encoded_s)

        value_estimate = value_estimate.item()

        if leaf.game.check_win() is not None: # if checkmate
            leaf.backup(value_estimate); continue

        child_priors = child_priors.detach().cpu().numpy().reshape(-1)
        leaf.expand(child_priors) # need to make sure valid moves
        leaf.backup(value_estimate)

    return np.argmax(root.child_number_visits), root.get_policy()

def MCTS_self_play(iter, num_games, chessnet):
    if not os.path.exists('datasets/iter{}'.format(iter)):
        os.makedirs('datasets/iter{}'.format(iter))

    for n in tqdm(range(num_games)):
        board = Board()
        checkmate = False
        dataset = []
        value = 0
        move_count = 0

        while not checkmate:
            best_move, policy = UCT_search(board, 500, chessnet)

            encoded_s = board.encode_board()
            draw_counter = 0
            for s, _ in reversed(dataset):
                if np.array_equal(encoded_s[:16], s[:16]):
                    draw_counter += 1
                if draw_counter >= 3: break

            if draw_counter >= 3: break

            dataset.append([encoded_s, policy])
            print("=============")
            print("move_count = {} draw_counter = {}".format(move_count, draw_counter))
            print(board)
            print("best_move = {} ({})".format(board.decode_move(best_move), best_move))
            board.move_chess(best_move)

            win_status = board.check_win()
            if win_status is not None:
                if win_status == Role.BLACK:
                    value = -1
                else:
                    value = 1
                checkmate = True
            move_count += 1

        print("iter {} checkmate {}".format(iter, checkmate))
        if not checkmate: continue
        dataset_pv = []
        for idx, data in enumerate(dataset):
            s, p = data
            if idx == 0:
                dataset_pv.append([s, p, 0])
            else:
                dataset_pv.append([s, p, value])

        with open('./datasets/iter{}/dataset_{}.pkl'.format(iter, int(time.time())), 'wb') as f:
            pickle.dump(dataset_pv, f)


