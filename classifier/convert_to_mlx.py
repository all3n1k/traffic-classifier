#!/usr/bin/env python3
"""
Convert ONNX model to MLX weights format.

This script reads the ONNX model and exports weights in numpy format
that can be loaded directly by the MLX server.

Usage:
    python convert_to_mlx.py
"""

import numpy as np
import onnx
from onnx import numpy_helper

def convert_onnx_to_mlx(onnx_path='TrafficClassifier.onnx', output_path='traffic_classifier.npz'):
    print(f"Loading ONNX model from {onnx_path}...")
    model = onnx.load(onnx_path)
    
    weights = {}
    
    for initializer in model.graph.initializer:
        name = initializer.name
        weight = numpy_helper.to_array(initializer)
        weights[name] = weight
        print(f"  {name}: {weight.shape}")
    
    print(f"Saving to {output_path}...")
    np.savez(output_path, **weights)
    print("Done!")
    
    return output_path

if __name__ == '__main__':
    import os
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    convert_onnx_to_mlx()
