#!/usr/bin/env python3
"""
ML Inference Server for Traffic Classifier

Loads the ONNX model and provides gRPC-style API for classification.
The Rust backend communicates with this server via JSON over HTTP.

Usage:
    python ml_server.py --model TrafficClassifier.onnx --port 50051

API:
    POST /classify
    Body: {"features": [src_port, dst_port, protocol, packet_size, payload_size, tcp_flags]}
    Response: {"class_id": 0, "class_name": "HTTP", "confidence": 0.95}
"""

import argparse
import json
import numpy as np
import onnxruntime as ort
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import parse_qs

CLASS_NAMES = ["HTTP", "HTTPS", "SSH", "FTP", "DNS", "SMTP", "MySQL", "PostgreSQL", "Redis", "MongoDB", "HTTP-Alt", "Unknown"]
NUM_CLASSES = len(CLASS_NAMES)

class MLServer:
    def __init__(self, model_path: str):
        print(f"Loading ONNX model from {model_path}...")
        self.session = ort.InferenceSession(model_path, providers=['CPUExecutionProvider'])
        self.input_name = self.session.get_inputs()[0].name
        self.output_name = self.session.get_outputs()[0].name
        print(f"Model loaded. Input: {self.input_name}, Output: {self.output_name}")
    
    def classify(self, features: list) -> dict:
        """Classify a single packet's features."""
        # Normalize features
        normalized = [
            features[0] / 65535.0,  # src_port
            features[1] / 65535.0,  # dst_port
            features[2] / 255.0,    # protocol
            features[3] / 1500.0,   # packet_size
            features[4] / 1400.0,   # payload_size
            features[5] / 255.0,   # tcp_flags
        ]
        
        input_data = np.array([normalized], dtype=np.float32)
        outputs = self.session.run([self.output_name], {self.input_name: input_data})
        
        probabilities = outputs[0][0]
        class_id = int(np.argmax(probabilities))
        confidence = float(probabilities[class_id])
        
        return {
            "class_id": class_id,
            "class_name": CLASS_NAMES[class_id],
            "confidence": confidence
        }
    
    def batch_classify(self, features_batch: list) -> list:
        """Classify multiple packets at once."""
        normalized_batch = []
        for features in features_batch:
            normalized = [
                features[0] / 65535.0,
                features[1] / 65535.0,
                features[2] / 255.0,
                features[3] / 1500.0,
                features[4] / 1400.0,
                features[5] / 255.0,
            ]
            normalized_batch.append(normalized)
        
        input_data = np.array(normalized_batch, dtype=np.float32)
        outputs = self.session.run([self.output_name], {self.input_name: input_data})
        
        probabilities = outputs[0]
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


class RequestHandler(BaseHTTPRequestHandler):
    ml_server = None
    
    def do_GET(self):
        if self.path == "/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"status": "healthy"}).encode())
        elif self.path == "/":
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            self.wfile.write(b"ML Inference Server - POST to /classify")
        else:
            self.send_response(404)
            self.end_headers()
    
    def do_POST(self):
        if self.path == "/classify":
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length)
            
            try:
                data = json.loads(body)
                features = data.get("features", [])
                
                if not features:
                    raise ValueError("Missing 'features' in request")
                
                result = self.ml_server.classify(features)
                
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps(result).encode())
                
            except Exception as e:
                self.send_response(400)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode())
        
        elif self.path == "/batch_classify":
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length)
            
            try:
                data = json.loads(body)
                features_batch = data.get("features_batch", [])
                
                if not features_batch:
                    raise ValueError("Missing 'features_batch' in request")
                
                results = self.ml_server.batch_classify(features_batch)
                
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"results": results}).encode())
                
            except Exception as e:
                self.send_response(400)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode())
        
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        print(f"[ML Server] {args[0]}")


def main():
    parser = argparse.ArgumentParser(description="ML Inference Server for Traffic Classifier")
    parser.add_argument("--model", default="TrafficClassifier.onnx", help="Path to ONNX model")
    parser.add_argument("--port", type=int, default=50051, help="Server port")
    parser.add_argument("--host", default="127.0.0.1", help="Server host")
    args = parser.parse_args()
    
    # Load model
    ml_server = MLServer(args.model)
    RequestHandler.ml_server = ml_server
    
    # Start server
    server = HTTPServer((args.host, args.port), RequestHandler)
    print(f"\nML Inference Server running on http://{args.host}:{args.port}")
    print("Endpoints:")
    print("  GET  /health     - Health check")
    print("  POST /classify   - Single classification")
    print("  POST /batch_classify - Batch classification")
    print("\nPress Ctrl+C to stop\n")
    
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.shutdown()


if __name__ == "__main__":
    main()