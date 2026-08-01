import React, { useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

export default function App() {
  const [msg, setMsg] = useState<string | null>(null)

  async function launch() {
    try {
      const res = await invoke('launch_dashboard', { dashboard_id: 'quantum-dashboard', workspace: 'research' })
      setMsg(String(res))
    } catch (e) {
      setMsg(String(e))
    }
  }

  return (
    <div style={{ padding: 20 }}>
      <h1>Quantum Browser Platform — Tauri UI</h1>
      <p>Use this UI to call native commands (launch dashboards, open browsers, manage profiles).</p>
      <button onClick={launch}>Launch Quantum Dashboard</button>
      {msg && <pre style={{ marginTop: 16 }}>{msg}</pre>}
    </div>
  )
}
