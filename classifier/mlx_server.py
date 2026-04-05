#!/usr/bin/env python3
"""
MLX Inference Server for Traffic Classifier

Apple Silicon optimized ML inference using MLX framework.
Faster inference on M1/M2/M3/M4/M5 chips.

Usage:
    pip install mlx
    python mlx_server.py --model traffic_classifier.npz --port 50052

API:
    POST /classify
    Body: {"features": [src_port, dst_port, protocol, packet_size, payload_size, tcp_flags]}
    Response: {"class_id": 0, "class_name": "HTTP", "confidence": 0.95}
"""

import argparse
import json
import time
import numpy as np

try:
    import mlx.core as mx
    import mlx.nn as nn
    MLX_AVAILABLE = True
except ImportError:
    MLX_AVAILABLE = False
    print("Warning: MLX not available. Install with: pip install mlx")
    exit(1)

from http.server import HTTPServer, BaseHTTPRequestHandler

CLASS_NAMES = ["HTTP", "HTTPS", "SSH", "FTP", "DNS", "SMTP", "MySQL", "PostgreSQL", "Redis", "MongoDB", "HTTP-Alt", "Unknown"]
NUM_CLASSES = len(CLASS_NAMES)


class TrafficClassifierMLX(nn.Module):
    def __init__(self, input_dim=6, hidden_dim=64, num_classes=NUM_CLASSES):
        super().__init__()
        self.layer1 = nn.Linear(input_dim, hidden_dim)
        self.relu1 = nn.ReLU()
        self.dropout1 = nn.Dropout(0.2)
        self.layer2 = nn.Linear(hidden_dim, hidden_dim * 2)
        self.relu2 = nn.ReLU()
        self.dropout2 = nn.Dropout(0.2)
        self.layer3 = nn.Linear(hidden_dim * 2, num_classes)
    
    def __call__(self, x):
        x = self.layer1(x)
        x = self.relu1(x)
        x = self.dropout1(x)
        x = self.layer2(x)
        x = self.relu2(x)
        x = self.dropout2(x)
        x = self.layer3(x)
        return x


class MLXServer:
    def __init__(self, weights_path: str = None):
        print(f"Initializing MLX...")
        
        self.model = TrafficClassifierMLX(input_dim=6, hidden_dim=64, num_classes=NUM_CLASSES)
        
        if weights_path and weights_path.endswith('.npz'):
            print(f"Loading weights from {weights_path}...")
            weights = np.load(weights_path)
            self.load_weights(weights)
        else:
            print("No weights file provided, using random initialization")
        
        self.model.eval()
        print("MLX model ready")
    
    def load_weights(self, npz_weights):
        """Load weights from numpy archive into MLX model."""
        weight_map = {
            'layers.0.weight': ('layer1', 'weight'),
            'layers.0.bias': ('layer1', 'bias'),
            'layers.3.weight': ('layer2', 'weight'),
            'layers.3.bias': ('layer2', 'bias'),
            'layers.6.weight': ('layer3', 'weight'),
            'layers.6.bias': ('layer3', 'bias'),
        }
        
        state = {}
        for key, value in npz_weights.items():
            if key in weight_map:
                layer, param_name = weight_map[key]
                if layer not in state:
                    state[layer] = {}
                state[layer][param_name] = mx.array(value)
            else:
                state[key] = mx.array(value)
        
        self.model.update(state)
    
    def normalize_features(self, features):
        """Normalize features to [0, 1] range."""
        return [
            features[0] / 65535.0,
            features[1] / 65535.0,
            features[2] / 255.0,
            features[3] / 1500.0,
            features[4] / 1400.0,
            features[5] / 255.0,
        ]
    
    def classify(self, features: list) -> dict:
        """Classify a single packet's features."""
        normalized = self.normalize_features(features)
        input_data = mx.array([normalized], dtype=mx.float32)
        
        output = self.model(input_data)
        probabilities = mx.softmax(output, axis=1)
        probabilities = probabilities[0].tolist()
        
        class_id = int(np.argmax(probabilities))
        confidence = float(probabilities[class_id])
        
        return {
            "class_id": class_id,
            "class_name": CLASS_NAMES[class_id],
            "confidence": confidence,
            "all_probabilities": {CLASS_NAMES[i]: prob for i, prob in enumerate(probabilities)}
        }
    
    def batch_classify(self, features_batch: list) -> list:
        """Classify multiple packets at once."""
        normalized_batch = [self.normalize_features(f) for f in features_batch]
        input_data = mx.array(normalized_batch, dtype=mx.float32)
        
        output = self.model(input_data)
        probabilities = mx.softmax(output, axis=1)
        probabilities = probabilities.tolist()
        
        results = []
        for probs in probabilities:
            class_id = int(np.argmax(probs))
            confidence = float(probs[class_id])
            results.append({
                "class_id": class_id,
                "class_name": CLASS_NAMES[class_id],
                "confidence": confidence
            })
        
        return results


class RequestHandler:
    def __init__(self, mlx_server):
        self.server = mlx_server
    
    def handle(self, method, path, body):
        if method == "GET" and path == "/health":
            return 200, {"status": "healthy", "backend": "mlx"}
        
        if method == "GET" and path == "/":
            return 200, {"service": "MLX Traffic Classifier", "version": "1.0"}
        
        if method == "GET" and path == "/benchmark":
            start = time.time()
            for _ in range(1000):
                self.server.classify([12345, 443, 6, 1500, 1400, 24])
            elapsed = time.time() - start
            return 200, {
                "inferences_per_second": round(1000 / elapsed, 2),
                "avg_latency_ms": round(elapsed * 1000 / 1000, 4)
            }
        
        if method == "POST" and path == "/classify":
            try:
                data = json.loads(body)
                features = data.get("features", [])
                if not features:
                    raise ValueError("Missing 'features'")
                result = self.server.classify(features)
                return 200, result
            except Exception as e:
                return 400, {"error": str(e)}
        
        if method == "POST" and path == "/batch_classify":
            try:
                data = json.loads(body)
                features_batch = data.get("features_batch", [])
                if not features_batch:
                    raise ValueError("Missing 'features_batch'")
                results = self.server.batch_classify(features_batch)
                return 200, {"results": results}
            except Exception as e:
                return 400, {"error": str(e)}
        
        return 404, {"error": "Not found"}


def create_server(mlx_server):
    from http.server import HTTPServer, BaseHTTPRequestHandler
    
    class Handler(BaseHTTPRequestHandler):
        request_handler = RequestHandler(mlx_server)
        
        def do_GET(self):
            status, body = self.request_handler.handle("GET", self.path, None)
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(body).encode())
        
        def do_POST(self):
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length) if content_length > 0 else b""
            status, body = self.request_handler.handle("POST", self.path, body)
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(body).encode())
        
        def log_message(self, format, *args):
            print(f"[MLX Server] {args[0]}")
    
    return Handler


def main():
    parser = argparse.ArgumentParser(description="MLX Inference Server for Traffic Classifier")
    parser.add_argument("--model", default="traffic_classifier.npz", help="Path to weights file (.npz)")
    parser.add_argument("--port", type=int, default=50052, help="Server port")
    parser.add_argument("--host", default="127.0.0.1", help="Server host")
    args = parser.parse_args()
    
    mlx_server = MLXServer(args.model)
    Handler = create_server(mlx_server)
    
    server = HTTPServer((args.host, args.port), Handler)
    print(f"\nMLX Inference Server running on http://{args.host}:{args.port}")
    print("Apple Silicon optimized inference")
    print("Endpoints:")
    print("  GET  /health         - Health check")
    print("  GET  /benchmark      - Run inference benchmark")
    print("  POST /classify       - Single classification")
    print("  POST /batch_classify - Batch classification")
    print("\nPress Ctrl+C to stop\n")
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.shutdown()


if __name__ == "__main__":
    main()
