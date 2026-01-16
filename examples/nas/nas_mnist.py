#!/usr/bin/env python3
"""
Neural Architecture Search (NAS) with ArqonHPO
==============================================
Uses Zero-Cost Proxy (Synflow) to evaluate architectures without training.
This is Phase 28 of the ArqonHPO roadmap.
"""

import json
import time
import os
import sys

# Force CPU for stability
os.environ["CUDA_VISIBLE_DEVICES"] = ""

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F
from torch.utils.data import DataLoader
from torchvision import datasets, transforms

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from arqonhpo import ArqonSolver

# =============================================================================
# CONFIGURATION
# =============================================================================
NAS_BUDGET = 100  # Number of architectures to evaluate
VALIDATION_EPOCHS = 5  # Epochs to train the best architecture
SEED = 42

torch.manual_seed(SEED)
np.random.seed(SEED)


# =============================================================================
# DYNAMIC CNN BUILDER
# =============================================================================
class DynamicCNN(nn.Module):
    """
    A CNN with configurable architecture.
    
    Parameters:
        num_layers: int [2-5] - Number of conv layers
        channels: list[int] - Channel sizes for each layer
        kernel_size: int [3,5,7] - Kernel size for all conv layers
        use_batchnorm: bool - Whether to use batch normalization
        dropout_rate: float [0-0.5] - Dropout rate
    """
    
    def __init__(self, num_layers, channels, kernel_size, use_batchnorm, dropout_rate):
        super().__init__()
        
        self.num_layers = num_layers
        layers = []
        in_channels = 1  # MNIST is grayscale
        
        for i in range(num_layers):
            out_channels = channels[min(i, len(channels) - 1)]
            
            # Conv layer
            layers.append(nn.Conv2d(in_channels, out_channels, kernel_size, padding=kernel_size // 2))
            
            # Optional BatchNorm
            if use_batchnorm:
                layers.append(nn.BatchNorm2d(out_channels))
            
            # ReLU + MaxPool
            layers.append(nn.ReLU())
            layers.append(nn.MaxPool2d(2))
            
            in_channels = out_channels
        
        self.features = nn.Sequential(*layers)
        
        # Calculate flattened size (MNIST 28x28, each pool halves)
        size = 28 // (2 ** num_layers)
        if size < 1:
            size = 1
        flat_size = in_channels * size * size
        
        self.classifier = nn.Sequential(
            nn.Flatten(),
            nn.Dropout(dropout_rate),
            nn.Linear(flat_size, 128),
            nn.ReLU(),
            nn.Dropout(dropout_rate),
            nn.Linear(128, 10),
        )
    
    def forward(self, x):
        x = self.features(x)
        x = self.classifier(x)
        return x


def build_model_from_config(config):
    """Build a DynamicCNN from ArqonHPO config."""
    num_layers = int(config["num_layers"])
    channels = [
        int(config["channels_0"]),
        int(config["channels_1"]),
        int(config["channels_2"]),
    ]
    kernel_size = int(config["kernel_size"])
    use_batchnorm = config["use_batchnorm"] > 0.5
    dropout_rate = config["dropout_rate"]
    
    return DynamicCNN(num_layers, channels, kernel_size, use_batchnorm, dropout_rate)


# =============================================================================
# SYNFLOW ZERO-COST PROXY
# =============================================================================
def synflow_score(model, input_shape=(1, 1, 28, 28)):
    """
    Compute Synflow score (Training-free architecture quality metric).
    
    Higher score = better architecture (more gradient flow capacity).
    Reference: https://arxiv.org/abs/2006.05467
    """
    try:
        model.eval()
        
        # Use ones as input
        x = torch.ones(input_shape, requires_grad=True)
        
        # Forward pass
        y = model(x)
        y = y.sum()
        
        # Backward pass
        y.backward()
        
        # Sum of (gradient * parameter) across all params
        score = 0.0
        for p in model.parameters():
            if p.grad is not None:
                score += (p.grad * p).sum().abs().item()
        
        return score
    except Exception as e:
        # Return 0 on any error (e.g., shape mismatch)
        return 0.0


# =============================================================================
# ARQONHPO NAS LOOP
# =============================================================================
def run_nas():
    """Run NAS using ArqonHPO to optimize architecture."""
    print("=" * 60)
    print("🏗️  ArqonHPO Neural Architecture Search (NAS)")
    print("=" * 60)
    print(f"Budget: {NAS_BUDGET} architectures | Proxy: Synflow (Zero-Cost)")
    print("-" * 60)
    
    # Define search space
    solver_config = {
        "seed": SEED,
        "budget": NAS_BUDGET,
        "bounds": {
            "num_layers": {"min": 2.0, "max": 5.0, "scale": "Linear"},
            "channels_0": {"min": 16.0, "max": 64.0, "scale": "Linear"},
            "channels_1": {"min": 32.0, "max": 128.0, "scale": "Linear"},
            "channels_2": {"min": 64.0, "max": 256.0, "scale": "Linear"},
            "kernel_size": {"min": 3.0, "max": 7.0, "scale": "Linear"},  # Will round to 3, 5, or 7
            "use_batchnorm": {"min": 0.0, "max": 1.0, "scale": "Linear"},
            "dropout_rate": {"min": 0.0, "max": 0.5, "scale": "Linear"},
        },
    }
    
    solver = ArqonSolver(json.dumps(solver_config))
    
    results = []
    start_time = time.time()
    
    for i in range(NAS_BUDGET):
        # Ask for candidate
        candidates = solver.ask()
        if not candidates:
            break
        
        config = candidates[0]
        
        # Round kernel size to valid values (3, 5, 7)
        raw_kernel = config["kernel_size"]
        if raw_kernel < 4:
            config["kernel_size"] = 3
        elif raw_kernel < 6:
            config["kernel_size"] = 5
        else:
            config["kernel_size"] = 7
        
        # Build and evaluate
        model = build_model_from_config(config)
        score = synflow_score(model)
        
        # Store result
        result = {
            "eval_id": i,
            "config": config.copy(),
            "score": score,
        }
        results.append(result)
        
        # Report to solver (minimize negative = maximize score)
        solver.tell(json.dumps([{
            "eval_id": i,
            "params": config,
            "value": -score,  # Minimize negative
            "cost": 1.0,
            "pruned": False,
        }]))
        
        if (i + 1) % 20 == 0:
            best_so_far = max(results, key=lambda x: x["score"])
            print(f"  [{i+1}/{NAS_BUDGET}] Best Synflow: {best_so_far['score']:.4f}")
    
    elapsed = time.time() - start_time
    
    # Sort by score
    results.sort(key=lambda x: x["score"], reverse=True)
    
    print("\n" + "=" * 60)
    print("🏆  TOP 5 ARCHITECTURES")
    print("=" * 60)
    
    for rank, res in enumerate(results[:5], 1):
        cfg = res["config"]
        print(f"\n#{rank} (Synflow: {res['score']:.4f})")
        print(f"   Layers: {int(cfg['num_layers'])}")
        print(f"   Channels: [{int(cfg['channels_0'])}, {int(cfg['channels_1'])}, {int(cfg['channels_2'])}]")
        print(f"   Kernel: {int(cfg['kernel_size'])}")
        print(f"   BatchNorm: {cfg['use_batchnorm'] > 0.5}")
        print(f"   Dropout: {cfg['dropout_rate']:.3f}")
    
    print(f"\n⏱️  Search Time: {elapsed:.2f}s ({NAS_BUDGET/elapsed:.1f} arch/sec)")
    
    # Return best config for validation
    return results[0]["config"], results


# =============================================================================
# VALIDATION: Train the Best Architecture
# =============================================================================
def validate_architecture(config):
    """Train the best architecture on MNIST to verify quality."""
    print("\n" + "=" * 60)
    print("🧪  VALIDATION: Training Best Architecture")
    print("=" * 60)
    
    # Load MNIST
    transform = transforms.Compose([
        transforms.ToTensor(),
        transforms.Normalize((0.1307,), (0.3081,))
    ])
    
    train_dataset = datasets.MNIST('./data', train=True, download=True, transform=transform)
    test_dataset = datasets.MNIST('./data', train=False, transform=transform)
    
    train_loader = DataLoader(train_dataset, batch_size=64, shuffle=True)
    test_loader = DataLoader(test_dataset, batch_size=1000, shuffle=False)
    
    # Build model
    model = build_model_from_config(config)
    optimizer = torch.optim.Adam(model.parameters(), lr=0.001)
    criterion = nn.CrossEntropyLoss()
    
    # Train
    model.train()
    for epoch in range(VALIDATION_EPOCHS):
        total_loss = 0
        for batch_idx, (data, target) in enumerate(train_loader):
            optimizer.zero_grad()
            output = model(data)
            loss = criterion(output, target)
            loss.backward()
            optimizer.step()
            total_loss += loss.item()
        
        avg_loss = total_loss / len(train_loader)
        print(f"  Epoch {epoch+1}/{VALIDATION_EPOCHS}: Loss = {avg_loss:.4f}")
    
    # Test
    model.eval()
    correct = 0
    total = 0
    with torch.no_grad():
        for data, target in test_loader:
            output = model(data)
            pred = output.argmax(dim=1, keepdim=True)
            correct += pred.eq(target.view_as(pred)).sum().item()
            total += len(target)
    
    accuracy = 100 * correct / total
    print(f"\n🎯  Test Accuracy: {accuracy:.2f}%")
    
    return accuracy


# =============================================================================
# MAIN
# =============================================================================
def main():
    # Phase 1: NAS
    best_config, all_results = run_nas()
    
    # Phase 2: Validate best architecture
    accuracy = validate_architecture(best_config)
    
    # Summary
    print("\n" + "=" * 60)
    print("📊  FINAL SUMMARY")
    print("=" * 60)
    print(f"Architectures Searched: {NAS_BUDGET}")
    print(f"Best Synflow Score: {all_results[0]['score']:.4f}")
    print(f"Best Architecture Accuracy: {accuracy:.2f}%")
    print("=" * 60)
    
    # Save results
    with open("nas_results.json", "w") as f:
        json.dump({
            "best_config": best_config,
            "best_accuracy": accuracy,
            "all_results": all_results[:10],  # Top 10
        }, f, indent=2)
    print("\nResults saved to nas_results.json")


if __name__ == "__main__":
    main()
