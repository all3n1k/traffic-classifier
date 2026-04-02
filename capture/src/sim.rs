use capture::capture::{PacketCapture, PacketFeatures};
use std::time::{Duration, SystemTime};
use rand::Rng;

fn simulate_packet() -> (PacketFeatures, Vec<u8>) {
    let mut rng = rand::thread_rng();
    
    let ports = [80, 443, 22, 21, 53, 25, 3306, 5432, 6379, 8080, 3000, 8000, 22, 22, 80, 80, 443, 443];
    let protocols = [0, 0, 0, 1, 1, 2]; 
    let sizes = [64, 128, 256, 512, 1024, 1400];
    
    let src_port: u16 = rng.gen_range(1024..65535);
    let dst_port = ports[rng.gen_range(0..ports.len())];
    let protocol = protocols[rng.gen_range(0..protocols.len())];
    let packet_size = sizes[rng.gen_range(0..sizes.len())] as u32;
    
    let payload_size = if packet_size > 60 { 
        (packet_size - 60) as usize 
    } else { 
        0 
    };
    
    let tcp_flags = match dst_port {
        80 | 443 | 8080 => 0x18,
        22 => 0x18,
        21 => 0x02,
        _ => 0x10,
    };
    
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    
    let features = PacketFeatures {
        src_port,
        dst_port,
        protocol,
        packet_size,
        payload_size: payload_size as u32,
        tcp_flags,
        timestamp_us: timestamp.as_micros() as u64,
    };
    
    let mut data = vec![0u8; packet_size as usize];
    data[0..14].copy_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x0c, 0x29, 0x00, 0x00, 0x00, 0x08, 0x00]);
    
    (features, data)
}

fn main() {
    println!("Starting packet simulation (for testing without pcap)...");
    
    let mut capture = PacketCapture::new("simulator".to_string()).unwrap();
    
    let mut count = 0;
    loop {
        let (features, _) = simulate_packet();
        
        let class = match features.dst_port {
            80 => "HTTP",
            443 => "HTTPS",
            22 => "SSH",
            21 => "FTP",
            53 => "DNS",
            25 | 587 | 465 => "SMTP",
            3306 => "MySQL",
            5432 => "PostgreSQL",
            6379 => "Redis",
            8080 | 8000 | 3000 => "HTTP-Alt",
            _ => "Unknown",
        };
        
        count += 1;
        if count % 10 == 0 {
            println!("[{}] Port: {} -> {}", count, features.dst_port, class);
        }
        
        if count >= 100 {
            break;
        }
        
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    println!("Simulated {} packets", count);
}