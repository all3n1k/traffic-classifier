#!/usr/bin/env python3
"""
Parse tcpdump text output and extract features for training.
"""

import re
import numpy as np
from collections import Counter

CLASS_NAMES = ["HTTP", "HTTPS", "SSH", "FTP", "DNS", "SMTP", "MySQL", "PostgreSQL", "Redis", "MongoDB", "HTTP-Alt", "Unknown"]

def port_to_label(port):
    port_map = {
        80: 0, 443: 1, 22: 2, 21: 3, 53: 4,
        25: 5, 587: 5, 465: 5, 110: 6, 995: 6,
        143: 7, 993: 7, 3306: 8, 5432: 9, 6379: 10, 27017: 10,
        8080: 11, 8000: 11, 3000: 11, 8443: 11, 9000: 11,
        5353: 11,  # mDNS
    }
    return port_map.get(port, 11)

def parse_tcpdump_line(line):
    """Parse a tcpdump line and extract features."""
    # Pattern: IP IP.src.port > IP.dst.port: flags
    # Examples:
    #   192.168.1.1.443 > 192.168.1.2.52341: Flags [P.], seq 1:100, ack 1, win 502
    #   52.201.147.143.443 > 192.168.1.1.41641: Flags [.], ack 12345, win 502
    #   100.69.199.68.53 > 100.100.100.100.50769: 5727+ A? something.com.
    
    # Skip non-IP lines
    if '>' not in line:
        return None
    
    # Extract IPs and ports
    # Pattern for: IP.port > IP.port
    match = re.search(r'(\d+\.\d+\.\d+\.\d+)\.(\d+)\s*>\s*(\d+\.\d+\.\d+\.\d+)\.(\d+)', line)
    if not match:
        return None
    
    src_ip, src_port, dst_ip, dst_port = match.groups()
    src_port = int(src_port)
    dst_port = int(dst_port)
    
    # Determine protocol from line
    proto = 6  # TCP default
    if 'UDP' in line or '.53' in line:
        proto = 17  # UDP
    
    # Extract packet size if available
    size = 64  # default
    size_match = re.search(r'length\s*(\d+)', line)
    if size_match:
        size = int(size_match.group(1))
    
    # Extract TCP flags
    tcp_flags = 0
    if 'Flags [' in line:
        flags_str = re.search(r'Flags \[([^\]]+)\]', line)
        if flags_str:
            flags = flags_str.group(1)
            if 'S' in flags and 'S.' not in flags.replace('SYN', ''):
                tcp_flags |= 0x02
            if 'P' in flags or 'PSH' in flags:
                tcp_flags |= 0x08
            if 'A' in flags and 'ACK' not in flags.replace('A', '') and '.' not in flags:
                tcp_flags |= 0x10
            if 'F' in flags:
                tcp_flags |= 0x01
            if 'R' in flags:
                tcp_flags |= 0x04
    
    # Filter: skip local to local (noise)
    if src_ip.startswith('192.168.') and dst_ip.startswith('192.168.'):
        return None
    if src_ip.startswith('127.') or dst_ip.startswith('127.'):
        return None
    if src_ip.startswith('100.'):
        return None
    
    return [src_port, dst_port, proto, size, 0, tcp_flags]

def main():
    print("Parsing tcpdump output...")
    
    features = []
    labels = []
    ports_seen = Counter()
    
    with open('/Users/neo/traffic_raw.txt', 'r') as f:
        for line in f:
            feat = parse_tcpdump_line(line)
            if feat:
                features.append(feat)
                label = port_to_label(feat[1])  # dst_port
                labels.append(label)
                ports_seen[feat[1]] += 1
    
    features = np.array(features)
    labels = np.array(labels)
    
    print(f"Extracted {len(features)} samples")
    
    print("\n=== Top Destination Ports ===")
    for port, count in ports_seen.most_common(20):
        label = CLASS_NAMES[port_to_label(port)]
        print(f"  {port:5} - {count:8,} - {label}")
    
    # Normalize
    normalized = np.column_stack([
        features[:, 0] / 65535.0,
        features[:, 1] / 65535.0,
        features[:, 2] / 255.0,
        features[:, 3] / 1500.0,
        features[:, 4] / 1400.0,
        features[:, 5] / 255.0,
    ])
    
    # Save
    np.savez('real_traffic_data.npz', X=normalized, y=labels)
    print(f"\nSaved {len(features)} samples to real_traffic_data.npz")

if __name__ == '__main__':
    main()
