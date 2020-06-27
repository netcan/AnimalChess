from tqdm import tqdm
import torch, os, pickle, datetime
import torch.optim as optim
import matplotlib.pyplot as plt
import numpy as np
from torch.utils.data import Dataset, DataLoader
from alpha_zero_net import AlphaLoss
from utils import load_net

class BoardDataset(Dataset):
    def __init__(self, dataset):
        self.dataset = dataset
    def __len__(self):
        return len(self.dataset)
    def __getitem__(self, idx):
        return self.dataset[idx][0], self.dataset[idx][1], self.dataset[idx][2]

def load_dataset(iter):
    data_path = 'datasets/iter{}/'.format(iter)
    datasets = []
    for file in os.listdir(data_path):
        filename = os.path.join(data_path, file)
        with open(filename, 'rb') as f:
            datasets.extend(pickle.load(f))
    datasets = np.array(datasets)
    return datasets


def train(iter, epoch_start=0, epoch_stop=100, seed=0):
    torch.manual_seed(seed)
    net = load_net(iter)
    net.train()

    cuda = torch.cuda.is_available()

    criterion = AlphaLoss()
    optimizer = optim.Adam(net.parameters(), lr=0.001)
    scheduler = optim.lr_scheduler.MultiStepLR(optimizer, milestones=[100,200,300,400], gamma=0.77)

    train_set = BoardDataset(load_dataset(iter))
    train_loader = DataLoader(train_set, batch_size=30, shuffle=True, num_workers=0, pin_memory=False)
    losses_per_epoch = []
    for epoch in tqdm(range(epoch_start, epoch_stop)):
        total_loss = 0.0
        losses_per_batch = []
        for i,data in enumerate(train_loader):
            state, policy, value = data
            state, policy, value = state.float(), policy.float(), value.float()
            if cuda:
                state, policy, value = state.cuda(), policy.cuda(), value.cuda()
            optimizer.zero_grad()
            policy_pred, value_pred = net(state) # policy_pred = torch.Size([batch, 4 * 9 * 7]) value_pred = torch.Size([batch, 1])
            loss = criterion(value_pred[:,0], value, policy_pred, policy)
            loss.backward()
            optimizer.step()
            total_loss += loss.item()
            if i % 10 == 9:    # print every 10 mini-batches of size = batch_size
                print('Process ID: %d [Epoch: %d, %5d/ %d points] total loss per batch: %.3f' %
                      (os.getpid(), epoch + 1, (i + 1)*30, len(train_set), total_loss/10))
                print("Policy:",policy[0].argmax().item(),policy_pred[0].argmax().item())
                print("Value:",value[0].item(),value_pred[0,0].item())
                losses_per_batch.append(total_loss/10)
                total_loss = 0.0
        losses_per_epoch.append(sum(losses_per_batch)/len(losses_per_batch))
        if len(losses_per_epoch) > 100:
            if abs(sum(losses_per_epoch[-4:-1])/3-sum(losses_per_epoch[-16:-13])/3) <= 0.01:
                break
        scheduler.step()

    fig = plt.figure()
    ax = fig.add_subplot()
    ax.scatter([e for e in range(1,epoch_stop+1,1)], losses_per_epoch)
    ax.set_xlabel("Epoch")
    ax.set_ylabel("Loss per batch")
    ax.set_title("Loss vs Epoch")
    print('Finished Training')
    plt.savefig(os.path.join("./model_data/", "Loss_vs_Epoch_%s.png" % datetime.datetime.today().strftime("%Y-%m-%d")))

    torch.save(net.state_dict(), './model_data/alpha_zero_net_iter{}.pth.tar'.format(iter + 1))
