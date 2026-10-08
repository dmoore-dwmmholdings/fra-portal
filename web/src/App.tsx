import { useEffect, useMemo, useState, type MouseEvent } from 'react'
import { loadChains, matches, treeRows, whys, type Chains, type Indexed } from './lib/chains'

/** View state lives in the URL, so every view is a link. */
interface View {
  q: string
  cls: string
  year: string
  sel: string
  focal: string
  depth: 'direct' | 'any'
  page: number
}

const DEFAULTS: View = { q: '', cls: '', year: '', sel: '', focal: 'crit.loc_inflight', depth: 'direct', page: 0 }
const PER_PAGE = 12
const NONE: Indexed[] = []
const NTSB_SEARCH = 'https://data.ntsb.gov/carol-main-public/basic-search'

function readView(): View {
  const p = new URLSearchParams(location.search)
  return {
    q: p.get('q') ?? DEFAULTS.q,
    cls: p.get('class') ?? DEFAULTS.cls,
    year: p.get('year') ?? DEFAULTS.year,
    sel: p.get('sel') ?? DEFAULTS.sel,
    focal: p.get('whys') ?? DEFAULTS.focal,
    depth: p.get('depth') === 'any' ? 'any' : 'direct',
    page: Math.max(0, Number(p.get('page') ?? 0) || 0),
  }
}

function href(v: View) {
  const p = new URLSearchParams()
  if (v.q) p.set('q', v.q)
  if (v.cls) p.set('class', v.cls)
  if (v.year) p.set('year', v.year)
  if (v.sel) p.set('sel', v.sel)
  if (v.focal !== DEFAULTS.focal) p.set('whys', v.focal)
  if (v.depth !== 'direct') p.set('depth', v.depth)
  if (v.page) p.set('page', String(v.page))
  const s = p.toString()
  return s ? `?${s}` : location.pathname
}

const pct = (n: number, d: number) => (d ? Math.round((100 * n) / d) : 0)

export default function App() {
  const [chains, setChains] = useState<{ data: Chains; recs: Indexed[] } | null>(null)
  const [error, setError] = useState('')
  const [view, setView] = useState<View>(readView)

  useEffect(() => {
    loadChains().then(setChains, (e: Error) => setError(e.message))
    const onPop = () => setView(readView())
    addEventListener('popstate', onPop)
    return () => removeEventListener('popstate', onPop)
  }, [])

  /** Typing replaces history; following a link pushes it. */
  const go = (patch: Partial<View>, push = true) => {
    const next = { ...view, ...patch }
    history[push ? 'pushState' : 'replaceState'](null, '', href(next))
    setView(next)
  }
  const link = (patch: Partial<View>) => ({
    href: href({ ...view, ...patch }),
    onClick: (e: MouseEvent) => {
      if (e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return
      e.preventDefault()
      go(patch)
    },
  })

  const recs = chains?.recs ?? NONE
  const base = useMemo(
    () => recs.filter((r) => (!view.cls || r.c === view.cls) && (!view.year || r.d.startsWith(view.year))),
    [recs, view.cls, view.year],
  )
  const list = useMemo(() => base.filter((r) => matches(r, view.q)), [base, view.q])
  const breakdown = useMemo(
    () => whys(base, view.focal, view.depth === 'any', chains?.data.labels ?? {}),
    [base, view.focal, view.depth, chains],
  )
  const classes = useMemo(() => {
    const n = new Map<string, number>()
    recs.forEach((r) => n.set(r.c, (n.get(r.c) ?? 0) + 1))
    return [...n].filter(([c]) => c).sort((a, b) => b[1] - a[1])
  }, [recs])
  const years = useMemo(() => [...new Set(recs.map((r) => r.d.slice(0, 4)))].filter(Boolean).sort().reverse(), [recs])

  if (error) return <div className="page"><p>could not load chains: {error}</p></div>
  if (!chains) return <div className="page"><p className="muted">loading chains…</p></div>

  const pages = Math.max(1, Math.ceil(list.length / PER_PAGE))
  const page = Math.min(view.page, pages - 1)
  const shown = list.slice(page * PER_PAGE, page * PER_PAGE + PER_PAGE)
  const selKey = (r: Indexed) => r.n || r.e
  const sel = recs.find((r) => selKey(r) === view.sel || r.e === view.sel) ?? list[0] ?? recs[0]
  const labels = chains.data.labels
  const focalLabel = labels[view.focal] ?? view.focal
  const rows = sel ? treeRows(sel.t) : []
  const barMax = 40

  return (
    <div className="page">
      <header className="top">
        <strong>FRA / chain viewer</strong>
        <span className="muted">
          {recs.length} analyses · NTSB avall · method {chains.data.methods.join(', ')}
        </span>
      </header>

      <form className="filters" onSubmit={(e) => e.preventDefault()}>
        <label>
          class
          <select value={view.cls} onChange={(e) => go({ cls: e.target.value, page: 0 })}>
            <option value="">all</option>
            {classes.map(([c, n]) => <option key={c} value={c}>{c} ({n})</option>)}
          </select>
        </label>
        <label>
          year
          <select value={view.year} onChange={(e) => go({ year: e.target.value, page: 0 })}>
            <option value="">all</option>
            {years.map((y) => <option key={y} value={y}>{y}</option>)}
          </select>
        </label>
        <label>
          node or text
          <input
            type="search"
            value={view.q}
            placeholder="crit.stall_spin, door, SR22"
            onChange={(e) => go({ q: e.target.value, page: 0 }, false)}
          />
        </label>
        {view.q && <a {...link({ q: '', page: 0 })}>clear</a>}
        <span className="muted">{list.length} of {recs.length} match</span>
      </form>

      <div className="cols">
        <nav className="list" aria-label="Analyses">
          <div className="muted row-head"><span>ntsb_no</span><span>date</span><span>aircraft</span></div>
          {shown.map((r) => (
            <a key={r.e + r.k} className={r === sel ? 'item on' : 'item'} aria-current={r === sel ? 'true' : undefined} {...link({ sel: selKey(r) })}>
              <span className="line"><span>{r.n || r.e}</span><span>{r.d}</span><span>{r.md || r.m}</span></span>
              <span className="muted">{r.h}</span>
            </a>
          ))}
          {list.length === 0 && <p className="muted">no chains match.</p>}
          <div className="pager muted">
            {list.length > 0 && <span>{page * PER_PAGE + 1}–{Math.min(list.length, (page + 1) * PER_PAGE)} of {list.length}</span>}
            {page > 0 && <a {...link({ page: page - 1 })}>prev</a>}
            {page < pages - 1 && <a {...link({ page: page + 1 })}>next</a>}
          </div>
        </nav>

        {sel && (
          <main className="detail">
            <strong>{sel.n || sel.e} · {sel.d}{sel.l && ` · ${sel.l}`}</strong>
            <dl className="facts">
              <dt>aircraft</dt><dd>{sel.m}{sel.c && ` (${sel.c})`}</dd>
              <dt>operation</dt><dd>{sel.p || '—'}</dd>
              <dt>outcome</dt><dd>{sel.o}{sel.f ? `, ${sel.f} killed` : ''}</dd>
              <dt>conditions</dt><dd>{sel.w || '—'}</dd>
              <dt>ev_id</dt><dd>{sel.e}{sel.k > 1 && ` / aircraft ${sel.k}`} · <a href={NTSB_SEARCH}>NTSB record</a></dd>
            </dl>

            <h2>probable cause (NTSB)</h2>
            <p className="cause">{sel.pc || '—'}</p>

            <h2>chain <span className="muted">[C] cause [F] contributing · select a node to filter and see its whys</span></h2>
            <div className="tree">
              {rows.map((t, i) => (
                <div key={i} className="node">
                  <span>
                    <span className="glyph">{t.pre}</span>
                    <a className={t.id === view.focal ? 'focal' : undefined} title={t.id} {...link({ q: t.id, focal: t.id, page: 0 })}>{t.label}</a>
                    {t.proposed && <span className="muted"> (proposed)</span>}
                  </span>
                  <span>{t.role && `[${t.role}]`}</span>
                  <span className="muted">{t.tier}</span>
                </div>
              ))}
            </div>

            <h2>
              whys of "{focalLabel}" <span className="muted">· {breakdown.N} of {base.length} chains · incidence</span>{' '}
              <span className="toggle">
                <a className={view.depth === 'direct' ? 'focal' : undefined} {...link({ depth: 'direct' })}>direct</a>
                <a className={view.depth === 'any' ? 'focal' : undefined} {...link({ depth: 'any' })}>any depth</a>
              </span>
            </h2>
            <div className="bars">
              {breakdown.rows.map((b) => {
                const p = pct(b.n, breakdown.N)
                return (
                  <div key={b.id} className="bar">
                    <a title={b.id} {...link({ q: b.id, focal: b.id, page: 0 })}>{b.label}</a>
                    <span className="num">{b.n}</span>
                    <span className="num">{p}%</span>
                    <span className="muted" aria-hidden="true">{'='.repeat(Math.max(1, Math.round((p / 100) * barMax)))}</span>
                  </div>
                )
              })}
              {breakdown.rows.length === 0 && <p className="muted">no whys recorded under this node.</p>}
            </div>
            <p className="muted">Narrative chains, filtered by class and year. Counts of accidents, not rates.</p>
          </main>
        )}
      </div>
    </div>
  )
}
