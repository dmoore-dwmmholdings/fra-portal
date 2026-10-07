import { useEffect, useState } from 'react'
import { listAnalyses, type Analysis } from './lib/analyses'

export default function App() {
  const [rows, setRows] = useState<Analysis[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [open, setOpen] = useState<string | null>(null)

  useEffect(() => {
    listAnalyses().then(setRows, (e: Error) => setError(e.message))
  }, [])

  return (
    <main>
      <header>
        <h1>Flight Report Analysis Portal</h1>
        <p>NTSB fixed-wing accident reports, broken down with What–Why Chain Analysis.</p>
      </header>

      {error && <p className="error">Could not load analyses: {error}</p>}
      {!rows && !error && <p>Loading…</p>}
      {rows?.length === 0 && <p>No analyses uploaded yet.</p>}

      <ul className="analyses">
        {rows?.map((a) => (
          <li key={a.id}>
            <button onClick={() => setOpen(open === a.id ? null : a.id)} aria-expanded={open === a.id}>
              <span className="mono">{a.ntsb_no ?? a.ev_id}</span>
              <span>{a.event_date}</span>
              <span>{[a.aircraft?.make, a.aircraft?.model].filter(Boolean).join(' ')}</span>
              {a.outcome && <span className={`tag ${a.outcome}`}>{a.outcome}</span>}
            </button>
            {open === a.id && (
              <div className="detail">
                <p>{a.summary}</p>
                {a.nodes && <p className="mono">{a.nodes.join(' → ')}</p>}
                <pre>{JSON.stringify(a.analysis, null, 2)}</pre>
              </div>
            )}
          </li>
        ))}
      </ul>
    </main>
  )
}
