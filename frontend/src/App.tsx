import { useState, useEffect, useRef } from 'react'
import { PieChart, Pie, Cell, BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, LineChart, Line } from 'recharts'

interface StatsMessage {
  total_packets: number
  packets_per_second: number
  classifications: Record<string, number>
  flows: FlowSummary[]
  timestamp: number
}

interface FlowSummary {
  dst_port: number
  protocol: string
  class_name: string
  packet_count: number
  byte_count: number
}

const COLORS = ['#58a6ff', '#3fb950', '#f85149', '#d29922', '#a371f7', '#79c0ff', '#ff7b72', '#7ee787']

function App() {
  const [stats, setStats] = useState<StatsMessage | null>(null)
  const [connected, setConnected] = useState(false)
  const [history, setHistory] = useState<{pps: number, time: number}[]>([])
  const wsRef = useRef<WebSocket | null>(null)

  useEffect(() => {
    const connect = () => {
      const ws = new WebSocket('ws://localhost:8080')
      
      ws.onopen = () => {
        setConnected(true)
        console.log('Connected to backend')
      }
      
      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          setStats(data)
          setHistory(prev => {
            const newHistory = [...prev, { pps: data.packets_per_second, time: Date.now() }]
            return newHistory.slice(-30)
          })
        } catch (e) {
          console.error('Parse error:', e)
        }
      }
      
      ws.onclose = () => {
        setConnected(false)
        setTimeout(connect, 2000)
      }
      
      ws.onerror = (e) => {
        console.error('WebSocket error:', e)
      }
      
      wsRef.current = ws
    }
    
    connect()
    
    return () => {
      wsRef.current?.close()
    }
  }, [])

  const pieData = stats ? Object.entries(stats.classifications).map(([name, value]) => ({
    name, value
  })) : []

  const barData = stats ? stats.flows.slice(0, 10).map(f => ({
    name: `${f.class_name}:${f.dst_port}`,
    packets: f.packet_count
  })) : []

  return (
    <div style={{ padding: '20px', maxWidth: '1400px', margin: '0 auto' }}>
      <header style={{ marginBottom: '30px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 style={{ fontSize: '28px', fontWeight: '600', color: '#fff' }}>Network Traffic Classifier</h1>
          <p style={{ color: '#8b949e', marginTop: '4px' }}>Real-time ML-powered packet analysis</p>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div style={{ 
            width: '10px', height: '10px', borderRadius: '50%',
            background: connected ? '#3fb950' : '#f85149'
          }} />
          <span style={{ color: connected ? '#3fb950' : '#f85149' }}>
            {connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </header>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '16px', marginBottom: '24px' }}>
        <StatCard 
          label="Total Packets" 
          value={stats?.total_packets.toLocaleString() || '0'} 
          icon="📦"
        />
        <StatCard 
          label="Packets/sec" 
          value={stats?.packets_per_second.toFixed(1) || '0'} 
          icon="⚡"
        />
        <StatCard 
          label="Protocols" 
          value={stats ? Object.keys(stats.classifications).length : '0'} 
          icon="🔬"
        />
        <StatCard 
          label="Active Flows" 
          value={stats?.flows.length.toString() || '0'} 
          icon="🌊"
        />
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '24px', marginBottom: '24px' }}>
        <div style={{ 
          background: '#161b22', borderRadius: '12px', padding: '20px',
          border: '1px solid #30363d'
        }}>
          <h3 style={{ marginBottom: '16px', color: '#fff' }}>Protocol Distribution</h3>
          <ResponsiveContainer width="100%" height={250}>
            <PieChart>
              <Pie
                data={pieData}
                cx="50%"
                cy="50%"
                innerRadius={60}
                outerRadius={100}
                dataKey="value"
                label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
              >
                {pieData.map((_, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip 
                contentStyle={{ background: '#21262d', border: '1px solid #30363d', borderRadius: '8px' }}
                itemStyle={{ color: '#c9d1d9' }}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>

        <div style={{ 
          background: '#161b22', borderRadius: '12px', padding: '20px',
          border: '1px solid #30363d'
        }}>
          <h3 style={{ marginBottom: '16px', color: '#fff' }}>Top Flows</h3>
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={barData} layout="vertical">
              <XAxis type="number" stroke="#8b949e" />
              <YAxis type="category" dataKey="name" stroke="#8b949e" width={100} />
              <Tooltip 
                contentStyle={{ background: '#21262d', border: '1px solid #30363d', borderRadius: '8px' }}
                itemStyle={{ color: '#c9d1d9' }}
              />
              <Bar dataKey="packets" fill="#58a6ff" radius={[0, 4, 4, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div style={{ 
        background: '#161b22', borderRadius: '12px', padding: '20px',
        border: '1px solid #30363d'
      }}>
        <h3 style={{ marginBottom: '16px', color: '#fff' }}>Throughput Over Time</h3>
        <ResponsiveContainer width="100%" height={200}>
          <LineChart data={history}>
            <XAxis dataKey="time" stroke="#8b949e" tickFormatter={() => ''} />
            <YAxis stroke="#8b949e" />
            <Tooltip 
              contentStyle={{ background: '#21262d', border: '1px solid #30363d', borderRadius: '8px' }}
              labelFormatter={() => ''}
              formatter={(value: number) => [`${value.toFixed(1)} pkt/s`, 'Packets/sec']}
            />
            <Line type="monotone" dataKey="pps" stroke="#58a6ff" strokeWidth={2} dot={false} />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  )
}

function StatCard({ label, value, icon }: { label: string; value: string; icon: string }) {
  return (
    <div style={{ 
      background: '#161b22', borderRadius: '12px', padding: '20px',
      border: '1px solid #30363d'
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
        <span style={{ fontSize: '24px' }}>{icon}</span>
        <div>
          <div style={{ fontSize: '28px', fontWeight: '700', color: '#fff' }}>{value}</div>
          <div style={{ fontSize: '14px', color: '#8b949e' }}>{label}</div>
        </div>
      </div>
    </div>
  )
}

export default App