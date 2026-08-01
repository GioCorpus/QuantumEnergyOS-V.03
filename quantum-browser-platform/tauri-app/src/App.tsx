import React, { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

type BrowserInfo = {
  id: string
  name: string
  installed: boolean
  path?: string
}

export default function App() {
  const [msg, setMsg] = useState<string | null>(null)
  const [browsers, setBrowsers] = useState<BrowserInfo[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [dashboardId, setDashboardId] = useState('quantum-dashboard')

  useEffect(() => {
    async function load() {
      try {
        const res = await invoke('list_browsers') as string
        const parsed: BrowserInfo[] = JSON.parse(res)
        setBrowsers(parsed)
        if (parsed.length > 0) setSelected(parsed[0].id)
      } catch (e) {
        setMsg(String(e))
      }
    }
    load()
  }, [])

  async function launch() {
    try {
      const payload: any = { dashboard_id: dashboardId, workspace: 'research' }
      if (selected) payload.browser_id = selected
      const res = await invoke('launch_dashboard', payload)
      setMsg(String(res))
    } catch (e) {
      setMsg(String(e))
    }
  }

  return (
    <div style={{ padding: 20 }}>
      <h1>Quantum Browser Platform — Tauri UI</h1>
      <p>Use this UI to call native commands (launch dashboards, open browsers, manage profiles).</p>

      <div style={{ marginBottom: 12 }}>
        <label>Dashboard ID: </label>
        <input value={dashboardId} onChange={e => setDashboardId(e.target.value)} />
      </div>

      <div>
        <h3>Detected Browsers</h3>
        {browsers.length === 0 && <div>No browsers detected</div>}
        {browsers.map(b => (
          <div key={b.id} style={{ marginBottom: 6 }}>
            <label>
              <input type="radio" name="browser" checked={selected === b.id} onChange={() => setSelected(b.id)} disabled={!b.installed} />
              {b.name} {b.installed ? '(installed)' : '(not installed)'} {b.path ? `- ${b.path}` : ''}
            </label>
          </div>
        ))}
      </div>

      <button onClick={launch} style={{ marginTop: 12 }}>Launch Dashboard</button>

      {msg && <pre style={{ marginTop: 16 }}>{msg}</pre>}
    </div>
  )
}
