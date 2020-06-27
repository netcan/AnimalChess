import torch, math, os
import numpy as np
import pickle, collections, time
from utils import load_net
from tqdm import tqdm
from animal_chess_pymodule import *
from alpha_zero_net import ChessNet
import torch.multiprocessing as mp

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
          return math.sqrt(self.number_visits) * (abs(self.child_priors)
                                        / (1 + self.child_number_visits))

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
        valid_child_priors = 0.75*valid_child_priors + 0.25*np.random.dirichlet(np.zeros([len(valid_child_priors)], dtype=np.float32)+192)
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

        #  if isinstance(self.parent, DummyNode): # root
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
        #  print('leaf step count = ', leaf.game.get_step_count());
        encoded_s = torch.from_numpy(np.array(leaf.game.encode_board())).float()
        if torch.cuda.is_available(): encoded_s = encoded_s.cuda()

        child_priors, value_estimate = net(encoded_s)

        value_estimate = value_estimate.item()

        if leaf.game.check_win() is not None: # if checkmate
            if leaf.game.check_win() == Role.RED:
                leaf.backup(1)
            else:
                leaf.backup(-1)
            continue

        child_priors = child_priors.detach().cpu().numpy().reshape(-1)
        leaf.expand(child_priors) # need to make sure valid moves
        leaf.backup(value_estimate)

    return np.argmax(root.child_number_visits), root.get_policy()

def worker(iter, num_games, net, workid = 0):
    for n in tqdm(range(num_games)):
        #  board = Board('2L3t/1d3c1/r1p1w1e/7/7/7/E1W1P1R/1C3D1/T3l2 w')
        board = Board()
        checkmate = False
        dataset = []
        value = 0
        move_count = 0

        while not checkmate:
            elapse = time.perf_counter()
            best_move, policy = UCT_search(board, 1400, net)
            elapse = time.perf_counter() - elapse

            encoded_s = np.array(board.encode_board())
            dataset.append([encoded_s, policy])

            print("=============")
            print("[workid:{}] elapse = {:.3f}s move_count = {} dup_times = {}".format(workid, elapse, move_count, board.get_dup_count()))
            print(board)
            print("best_move = {} ({})".format(board.decode_move(best_move), best_move))
            board.move_chess(best_move)

            win_status = board.check_win()
            if win_status is not None:
                print("[workid:{}] game end! dup_times = {}".format(workid, board.get_dup_count()))
                print(board)
                if win_status == Role.BLACK:
                    value = -1
                else:
                    value = 1
                checkmate = True
            move_count += 1

        print("iter {} checkmate {} value = {}".format(iter, checkmate, value))
        #  if not checkmate: continue
        dataset_pv = []
        for idx, data in enumerate(dataset):
            s, p = data
            if idx == 0:
                dataset_pv.append([s, p, 0])
            else:
                dataset_pv.append([s, p, value])

        with open('./datasets/iter{}/dataset_{}.pkl'.format(iter, int(time.time() * 1000)), 'wb') as f:
            pickle.dump(dataset_pv, f)

def MCTS_self_play(iter, num_games, workers = 1):
    if not os.path.exists('datasets/iter{}'.format(iter)):
        os.makedirs('datasets/iter{}'.format(iter))

    net = load_net(iter)
    if workers > 1:
        net.share_memory()
        mp.set_start_method("spawn",force=True)
    net.eval()


    if workers > 1:
        process = []
        for i in range(workers):
            w = mp.Process(target=worker, args=(iter, num_games, net, i))
            w.start()
            process.append(w)

        for w in process:
            w.join()
    else:
        worker(iter, num_games, net)

