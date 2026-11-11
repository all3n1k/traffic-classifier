"""Bounded HTTP input shared by the optional model servers."""

from http.server import BaseHTTPRequestHandler
import json
import math

MAX_BODY = 16_384
MAX_BATCH = 128
FEATURE_LIMITS = (65_535, 65_535, 255, 65_535, 65_535, 255)


def valid_features(values):
    return (
        isinstance(values, list)
        and len(values) == len(FEATURE_LIMITS)
        and all(type(value) in (int, float) and 0 <= value <= limit
                for value, limit in zip(values, FEATURE_LIMITS))
    )


class InferenceHandler(BaseHTTPRequestHandler):
    timeout = 5
    classifier = None
    backend = "onnx"

    def respond(self, status, payload):
        data = json.dumps(payload, allow_nan=False).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        if self.path in {"/", "/health"}:
            healthy = self.classifier is not None
            self.respond(200 if healthy else 503, {"status": "healthy" if healthy else "unavailable", "backend": self.backend})
        else:
            self.respond(404, {"error": "Not found"})

    def do_POST(self):
        if self.path not in {"/classify", "/batch_classify"}:
            self.respond(404, {"error": "Not found"})
            return
        if self.headers.get_content_type() != "application/json":
            self.respond(415, {"error": "JSON required"})
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
        except ValueError:
            length = 0
        if not 0 < length <= MAX_BODY:
            self.respond(413, {"error": "Invalid request size"})
            return
        try:
            raw = self.rfile.read(length)
            if len(raw) != length:
                raise ValueError
            body = json.loads(raw)
            if not isinstance(body, dict):
                raise ValueError
            batch = self.path == "/batch_classify"
            values = body.get("features_batch" if batch else "features")
            if batch:
                if not isinstance(values, list) or not 0 < len(values) <= MAX_BATCH or not all(valid_features(row) for row in values):
                    raise ValueError
            elif not valid_features(values):
                raise ValueError
        except (ValueError, TimeoutError, RecursionError):
            self.respond(400, {"error": "Invalid inference input"})
            return
        if self.classifier is None:
            self.respond(503, {"error": "Inference unavailable"})
            return
        try:
            result = {"results": self.classifier.batch_classify(values)} if batch else self.classifier.classify(values)
            rows = result["results"] if batch else [result]
            if not rows or not all(math.isfinite(row["confidence"]) for row in rows):
                raise ValueError
        except Exception:
            self.respond(503, {"error": "Inference unavailable"})
            return
        self.respond(200, result)
