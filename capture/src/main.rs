use capture::{PacketFeatures, start_capture};
use anyhow::Result;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting packet capture demo...");
    
    let device = "lo0";
    let (tx, mut rx) = mpsc::channel(100);
    
    let _ = tokio::spawn(async move {
        if let Err(e) = start_capture(device.to_string(), tx).await {
            eprintln!("Capture error: {}", e);
        }
    });
    
    tokio::spawn(async move {
        let mut count = 0;
        while let Some(output) = rx.recv().await {
            count += 1;
            if count % 10 == 0 {
                println!("[{}] Classified: {} (conf: {:.2})", 
                    count, output.class_name, output.confidence);
            }
            if count >= 100 {
                break;
            }
        }
        println!("Processed {} packets", count);
    });
    
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    Ok(())
}