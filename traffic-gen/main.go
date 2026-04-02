package main

import (
	"fmt"
	"math/rand"
	"net"
	"time"
)

func main() {
	rand.Seed(time.Now().UnixNano())
	
	ports := []int{80, 443, 22, 21, 53, 25, 3306, 5432, 6379, 8080, 3000, 8000}
	protocols := []string{"TCP", "UDP"}
	
	fmt.Println("Generating synthetic traffic...")
	
	for i := 0; i < 10000; i++ {
		srcPort := rand.Intn(64511) + 1024
		dstPort := ports[rand.Intn(len(ports))]
		protocol := protocols[rand.Intn(len(protocols))]
		
		packetSize := []int{64, 128, 256, 512, 1024, 1400}[rand.Intn(6)]
		
		_ = net.UDPAddr{
			IP:   []byte{127, 0, 0, 1},
			Port: srcPort,
		}
		
		if i%1000 == 0 {
			fmt.Printf("Generated %d packets (port %d, %s, %d bytes)\n", i, dstPort, protocol, packetSize)
		}
		
		time.Sleep(time.Microsecond * 100)
	}
	
	fmt.Println("Done generating traffic")
}