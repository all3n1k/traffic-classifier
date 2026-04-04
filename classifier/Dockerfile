# ML Inference Server Dockerfile
FROM python:3.11-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

COPY classifier/requirements.txt /app/
RUN pip install --no-cache-dir -r requirements.txt

COPY classifier/ml_server.py /app/
COPY classifier/TrafficClassifier.onnx /app/

ENV HOST=0.0.0.0
ENV PORT=50051

EXPOSE 50051

CMD ["python", "ml_server.py", "--host", "0.0.0.0", "--port", "50051"]
