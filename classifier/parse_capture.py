#!/usr/bin/env python3
"""
Parse pcap capture and extract features for model training.
Uses sampling for efficiency on large captures.
"""

import sys
from scapy.all import rdpcap, TCP, UDP, IP
import numpy as np
from collections import Counter

PROTOCOL_MAP = {6: 'TCP', 17: 'UDP', 1: 'ICMP'}
CLASS_NAMES = ["HTTP", "HTTPS", "SSH", "FTP", "DNS", "SMTP", "MySQL", "PostgreSQL", "Redis", "MongoDB", "HTTP-Alt", "Unknown"]

def port_to_label(port, protocol):
    port_map = {
        80: 0, 443: 1, 22: 2, 21: 3, 53: 4,
        25: 5, 587: 5, 465: 5, 110: 6, 995: 6,
        143: 7, 993: 7, 3306: 8, 5432: 9, 6379: 10, 27017: 10,
        8080: 11, 8000: 11, 3000: 11, 8443: 11, 9000: 11,
    }
    return port_map.get(port, 11)

def extract_features(pkt):
    if not pkt.haslayer(IP):
        return None
    
    src_port = dst_port = protocol = packet_size = payload_size = tcp_flags = 0
    
    if TCP in pkt:
        src_port, dst_port = pkt[TCP].sport, pkt[TCP].dport
        protocol = 6
        tcp_flags = int(pkt[TCP].flags) if pkt[TCP].flags else 0
    elif UDP in pkt:
        src_port, dst_port = pkt[UDP].sport, pkt[UDP].dport
        protocol = 17
    
    packet_size = len(pkt)
    payload_size = len(pkt.payload)
    
    return [src_port, dst_port, protocol, packet_size, payload_size, tcp_flags]

def main():
    pcap_file = sys.argv[1] if len(sys.argv) > 1 else '/Users/neo/traffic_capture.pcap'
    sample_rate = int(sys.argv[2]) if len(sys.argv) > 2 else 100  # 1 in 100 packets
    
    print(f"Loading {pcap_file} (sampling 1 in {sample_rate})...")
    
    features = []
    labels = []
    ports_seen = Counter()
    count = 0
    
    for pkt in rdpcap(pcap_file):
        count += 1
        if count % sample_rate != 0:
            continue
        
        feat = extract_features(pkt)
        if feat:
            features.append(feat)
            label = port_to_label(feat[1], feat[2])  # dst_port, protocol
            labels.append(label)
            ports_seen[(feat[1], feat[2])] += 1
        
        if count % 100000 == 0:
            print(f"  Processed {count:,} packets, extracted {len(features):,} samples...")
    
    features = np.array(features)
    labels = np.array(labels)
    
    print(f"\nExtracted {len(features):,} samples from {count:,} packets")
    
    print("\n=== Protocol Distribution ===")
    label_counts = Counter(labels)
    for label_id, count in sorted(label_counts.items(), key=lambda x: -x[1])[:15]:
        print(f"  {CLASS_NAMES[label_id]:12} - {count:8,} packets")
    
    # Normalize features
    normalized = np.column_stack([
        features[:, 0] / 65535.0,
        features[:, 1] / 65535.0,
        features[:, 2] / 255.0,
        features[:, 3] / 1500.0,
        features[:, 4] / 1400.0,
        features[:, 5] / 255.0,
    ])
    
    # Save
    output = 'real_traffic_data.npz'
    np.savez(output, features=normalized, labels=labels)
    print(f"\nSaved to {output}")
    
    # Also save raw for training
    raw_output = 'real_traffic_features.npz'
    np.savez(raw_output, X=normalized, y=labels)
    print(f"Saved raw to {raw_output}")

if __name__ == '__main__':
    main()
