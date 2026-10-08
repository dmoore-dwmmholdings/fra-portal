/** Mirrors functions/src/chains.ts (GET /api/chains). */
export type ChainNode = [id: string, label: string, role: '' | 'C' | 'F', proposed: 0 | 1, kids?: ChainNode[]]

export interface ChainRecord {
  e: string
  k: number
  n: string
  d: string
  o: string
  f: number | null
  l: string
  m: string
  md: string
  c: string
  p: string
  w: string
  h: string
  pc: string
  t: ChainNode
}

export interface Chains {
  generated: string
  methods: string[]
  labels: Record<string, string>
  records: ChainRecord[]
}

export interface Indexed extends ChainRecord {
  ids: Set<string>
  text: string
}

export async function loadChains(): Promise<{ data: Chains; recs: Indexed[] }> {
  const res = await fetch('/api/chains')
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  const data = (await res.json()) as Chains
  const recs = data.records.map((r) => {
    const ids = new Set<string>()
    let text = `${r.n} ${r.m} ${r.h} ${r.l}`
    walk(r.t, (n) => {
      ids.add(n[0])
      text += ` ${n[1]}`
    })
    return { ...r, ids, text: text.toLowerCase() }
  })
  return { data, recs }
}

export const kids = (n: ChainNode) => n[4] ?? []

export function walk(n: ChainNode, fn: (n: ChainNode) => void) {
  fn(n)
  for (const c of kids(n)) walk(c, fn)
}

/** Segment-aware prefix: "mech.fuel" covers "mech.fuel.starvation", not "mech.fuelx". */
export const hasPrefix = (id: string, p: string) => id === p || id.startsWith(p + '.')

const TIERS: Record<string, string> = {
  outcome: 'outcome', end: 'end', crit: 'critical', mech: 'mechanism', und: 'undetermined', act: 'act', latent: 'latent', env: 'context',
}
export const tierOf = (id: string) => TIERS[id.split('.')[0]] ?? ''

const NODE_QUERY = /^(outcome|end|crit|mech|und|act|latent|env)(\.[a-z0-9_]+)*$/

/** Node-id queries match a node on the chain; anything else is a text search. */
export function matches(r: Indexed, q: string) {
  q = q.trim()
  if (!q) return true
  if (NODE_QUERY.test(q)) {
    for (const id of r.ids) if (hasPrefix(id, q)) return true
    return false
  }
  return r.text.includes(q.toLowerCase())
}

export interface TreeRow {
  pre: string
  id: string
  label: string
  role: string
  tier: string
  proposed: boolean
}

/** Flatten a chain into rows with teletype connectors (├─ └─ │). */
export function treeRows(root: ChainNode): TreeRow[] {
  const rows: TreeRow[] = []
  const add = (n: ChainNode, pre: string, cont: string) => {
    rows.push({ pre, id: n[0], label: n[1], role: n[2], tier: tierOf(n[0]), proposed: n[3] === 1 })
    const k = kids(n)
    k.forEach((c, i) => {
      const last = i === k.length - 1
      add(c, cont + (last ? '└─ ' : '├─ '), cont + (last ? '   ' : '│  '))
    })
  }
  add(root, '', '')
  return rows
}

export interface WhyRow {
  id: string
  label: string
  n: number
}

/**
 * Whys of a focal node (spec §6.1, incidence): among records whose chain contains the focal node (by prefix),
 * how many show each why under it. `any` looks at every depth below, else direct whys only. Nodes inside the
 * focal bucket are passed through.
 */
export function whys(recs: Indexed[], focal: string, any: boolean, labels: Record<string, string>) {
  const count = new Map<string, number>()
  const label = new Map<string, string>()
  let N = 0
  for (const r of recs) {
    if (![...r.ids].some((id) => hasPrefix(id, focal))) continue
    N++
    const seen = new Set<string>()
    const collect = (c: ChainNode) => {
      if (hasPrefix(c[0], focal)) {
        kids(c).forEach(collect)
        return
      }
      seen.add(c[0])
      if (!label.has(c[0])) label.set(c[0], labels[c[0]] ?? c[1])
      if (any) kids(c).forEach(collect)
    }
    walk(r.t, (n) => {
      if (hasPrefix(n[0], focal)) kids(n).forEach(collect)
    })
    seen.forEach((id) => count.set(id, (count.get(id) ?? 0) + 1))
  }
  const rows: WhyRow[] = [...count]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .slice(0, 12)
    .map(([id, n]) => ({ id, n, label: label.get(id)! }))
  return { N, rows }
}
