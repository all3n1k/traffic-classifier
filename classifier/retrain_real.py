#!/usr/bin/env python3
"""
Retrain the traffic classifier with real captured data.
"""

import torch
import torch.nn as nn
from torch.utils.data import Dataset, DataLoader
import numpy as np
from sklearn.model_selection import train_test_split
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

def load_real_data():
    """Load captured traffic data."""
    data = np.load('real_traffic_data.npz')
    X = data['X']
    y = data['y']
    print(f"Loaded {len(X)} samples from real traffic")
    
    # Check distribution
    unique, counts = np.unique(y, return_counts=True)
    print("\nClass distribution:")
    for u, c in sorted(zip(unique, counts), key=lambda x: -x[1]):
        print(f"  {CLASS_NAMES[u]:12} - {c:5} samples")
    
    return X, y

def augment_data(X, y, target_per_class=2000):
    """Augment data to balance classes."""
    X_aug = [X]
    y_aug = [y]
    
    for class_id in range(NUM_CLASSES):
        mask = y == class_id
        class_X = X[mask]
        class_count = len(class_X)
        
        if class_count == 0:
            continue
        
        target = min(target_per_class, class_count * 10)
        
        if class_count < target:
            # Add noise to existing samples
            multiplier = int(np.ceil(target / class_count))
            for _ in range(multiplier - 1):
                noise = np.random.normal(0, 0.01, class_X.shape)
                augmented = np.clip(class_X + noise, 0, 1)
                X_aug.append(augmented)
                y_aug.append(np.full(len(class_X), class_id))
    
    return np.vstack(X_aug), np.concatenate(y_aug)

def train_model(X, y, epochs=20, batch_size=128, learning_rate=0.001):
    # Split data
    X_train, X_val, y_train, y_val = train_test_split(X, y, test_size=0.2, random_state=42)
    
    train_dataset = TrafficDataset(X_train, y_train)
    val_dataset = TrafficDataset(X_val, y_val)
    
    train_loader = DataLoader(train_dataset, batch_size=batch_size, shuffle=True)
    val_loader = DataLoader(val_dataset, batch_size=batch_size)
    
    model = TrafficClassifier(input_dim=6, hidden_dim=64, num_classes=NUM_CLASSES)
    criterion = nn.CrossEntropyLoss()
    optimizer = torch.optim.Adam(model.parameters(), lr=learning_rate)
    scheduler = torch.optim.lr_scheduler.ReduceLROnPlateau(optimizer, patience=3, factor=0.5)
    
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
            torch.save(model.state_dict(), 'traffic_classifier_real.pth')
        
        print(f"Epoch {epoch+1}/{epochs} - Loss: {train_loss/len(train_loader):.4f} - Val Acc: {val_acc:.4f}")
    
    print(f"Best validation accuracy: {best_val_acc:.4f}")
    return model

def export_to_onnx(model_path='traffic_classifier_real.pth', output_path='TrafficClassifier.onnx'):
    model = TrafficClassifier(input_dim=6, hidden_dim=64, num_classes=NUM_CLASSES)
    model.load_state_dict(torch.load(model_path, map_location='cpu'))
    model.eval()
    
    dummy_input = torch.randn(1, 6)
    torch.onnx.export(
        model, dummy_input, output_path,
        input_names=['input'], output_names=['output'],
        dynamic_axes={'input': {0: 'batch_size'}, 'output': {0: 'batch_size'}},
        opset_version=14
    )
    print(f"Model exported to {output_path}")

def export_to_mlx(model_path='traffic_classifier_real.pth', output_path='traffic_classifier_real.npz'):
    """Export to MLX format."""
    model = TrafficClassifier(input_dim=6, hidden_dim=64, num_classes=NUM_CLASSES)
    model.load_state_dict(torch.load(model_path, map_location='cpu'))
    model.eval()
    
    weights = {}
    for name, param in model.named_parameters():
        weights[name] = param.detach().numpy()
    
    np.savez(output_path, **weights)
    print(f"MLX weights exported to {output_path}")

if __name__ == '__main__':
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    
    print("="*50)
    print("Training with REAL captured traffic")
    print("="*50)
    
    X, y = load_real_data()
    X, y = augment_data(X, y)
    print(f"\nTotal samples after augmentation: {len(X)}")
    
    model = train_model(X, y, epochs=20)
    export_to_onnx()
    export_to_mlx()
    
    print("\nDone! Trained on your actual network traffic.")
