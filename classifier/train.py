import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import numpy as np
from sklearn.model_selection import train_test_split
import onnx
from onnx import helper, numpy_helper
import os

CLASS_NAMES = ["HTTP", "HTTPS", "SSH", "FTP", "DNS", "SMTP", "MySQL", "PostgreSQL", "Redis", "MongoDB", "HTTP-Alt", "Unknown"]
NUM_CLASSES = len(CLASS_NAMES)

class TrafficDataset(Dataset):
    def __init__(self, features, labels):
        self.features = torch.FloatTensor(features)
        self.labels = torch.LongTensor(labels)
    
    def __len__(self):
        return len(self.features)
    
    def __getitem__(self, idx):
        return self.features[idx], self.labels[idx]

class TrafficClassifier(nn.Module):
    def __init__(self, input_dim=6, hidden_dim=64, num_classes=NUM_CLASSES):
        super().__init__()
        self.layers = nn.Sequential(
            nn.Linear(input_dim, hidden_dim),
            nn.ReLU(),
            nn.Dropout(0.2),
            nn.Linear(hidden_dim, hidden_dim * 2),
            nn.ReLU(),
            nn.Dropout(0.2),
            nn.Linear(hidden_dim * 2, num_classes)
        )
    
    def forward(self, x):
        return self.layers(x)

def extract_features(packet_info):
    src_port = float(packet_info.get('src_port', 0))
    dst_port = float(packet_info.get('dst_port', 0))
    protocol = float(packet_info.get('protocol', 0))
    packet_size = float(packet_info.get('packet_size', 0))
    payload_size = float(packet_info.get('payload_size', 0))
    tcp_flags = float(packet_info.get('tcp_flags', 0))
    
    return [src_port / 65535.0, dst_port / 65535.0, protocol / 3.0,
            packet_size / 1500.0, payload_size / 1400.0, tcp_flags / 255.0]

def port_to_label(port):
    port_map = {
        80: 0, 443: 1, 22: 2, 21: 3, 53: 4,
        25: 5, 587: 5, 465: 5, 110: 6, 995: 6,
        143: 7, 993: 7, 3306: 8, 5432: 9, 6379: 10, 27017: 10,
        8080: 11, 8000: 11, 3000: 11
    }
    return port_map.get(port, 11)

def generate_synthetic_data(num_samples=50000):
    data = []
    labels = []
    
    configs = [
        (80, "HTTP", 0.15), (443, "HTTPS", 0.15), (22, "SSH", 0.10),
        (21, "FTP", 0.05), (53, "DNS", 0.10), (25, "SMTP", 0.05),
        (3306, "MySQL", 0.08), (5432, "PostgreSQL", 0.08),
        (6379, "Redis", 0.05), (8080, "HTTP-Alt", 0.08), (0, "Unknown", 0.11)
    ]
    
    for port, _, weight in configs:
        count = int(num_samples * weight)
        for _ in range(count):
            packet = {
                'src_port': np.random.randint(1024, 65535),
                'dst_port': port if port > 0 else np.random.randint(1, 65535),
                'protocol': 0 if port in [80, 443, 22, 21, 25, 3306, 5432, 8080] else (1 if port == 53 else 0),
                'packet_size': int(np.random.choice([64, 128, 256, 512, 1024, 1400])),
                'payload_size': int(np.random.choice([0, 32, 64, 128, 256, 1000])),
                'tcp_flags': 0x18 if port in [80, 443] else (0x02 if port == 22 else 0x10)
            }
            data.append(extract_features(packet))
            labels.append(port_to_label(packet['dst_port']))
    
    return np.array(data), np.array(labels)

def train_model(epochs=30, batch_size=256, learning_rate=0.001):
    print("Generating synthetic training data...")
    X, y = generate_synthetic_data(50000)
    
    X_train, X_val, y_train, y_val = train_test_split(X, y, test_size=0.2, random_state=42)
    
    train_dataset = TrafficDataset(X_train, y_train)
    val_dataset = TrafficDataset(X_val, y_val)
    
    train_loader = DataLoader(train_dataset, batch_size=batch_size, shuffle=True)
    val_loader = DataLoader(val_dataset, batch_size=batch_size)
    
    model = TrafficClassifier(input_dim=6, hidden_dim=64, num_classes=NUM_CLASSES)
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.Adam(model.parameters(), lr=learning_rate)
    scheduler = optim.lr_scheduler.ReduceLROnPlateau(optimizer, patience=3, factor=0.5)
    
    best_val_acc = 0.0
    
    for epoch in range(epochs):
        model.train()
        train_loss = 0.0
        for batch_x, batch_y in train_loader:
            optimizer.zero_grad()
            outputs = model(batch_x)
            loss = criterion(outputs, batch_y)
            loss.backward()
            optimizer.step()
            train_loss += loss.item()
        
        model.eval()
        val_correct = 0
        val_total = 0
        with torch.no_grad():
            for batch_x, batch_y in val_loader:
                outputs = model(batch_x)
                _, predicted = torch.max(outputs.data, 1)
                val_total += batch_y.size(0)
                val_correct += (predicted == batch_y).sum().item()
        
        val_acc = val_correct / val_total
        scheduler.step(train_loss)
        
        if val_acc > best_val_acc:
            best_val_acc = val_acc
            torch.save(model.state_dict(), 'classifier/traffic_classifier.pth')
        
        print(f"Epoch {epoch+1}/{epochs} - Loss: {train_loss/len(train_loader):.4f} - Val Acc: {val_acc:.4f}")
    
    print(f"Best validation accuracy: {best_val_acc:.4f}")
    return model

def export_to_onnx(model_path='classifier/traffic_classifier.pth', output_path='classifier/TrafficClassifier.onnx'):
    model = TrafficClassifier(input_dim=6, hidden_dim=64, num_classes=NUM_CLASSES)
    model.load_state_dict(torch.load(model_path, map_location='cpu'))
    model.eval()
    
    dummy_input = torch.randn(1, 6)
    
    torch.onnx.export(
        model,
        dummy_input,
        output_path,
        input_names=['input'],
        output_names=['output'],
        dynamic_axes={'input': {0: 'batch_size'}, 'output': {0: 'batch_size'}},
        opset_version=14
    )
    
    print(f"Model exported to {output_path}")

if __name__ == '__main__':
    os.makedirs('classifier', exist_ok=True)
    model = train_model(epochs=30)
    export_to_onnx()
    print("Training complete!")