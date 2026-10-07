import { collection, getDocs, limit, orderBy, query, type Timestamp } from 'firebase/firestore'
import { db } from './firebase'

/** Mirrors the `put_analysis` MCP tool input (functions/src/mcp.ts). */
export interface Analysis {
  id: string
  ev_id: string
  aircraft_key: number
  ntsb_no?: string
  event_date?: string
  outcome?: 'fatal' | 'serious' | 'minor' | 'none' | 'unknown'
  aircraft?: { make?: string; model?: string; class_id?: string; family_id?: string }
  summary: string
  nodes?: string[]
  analysis: Record<string, unknown>
  method_version: string
  analyst?: string
  updated_at?: Timestamp
}

export async function listAnalyses(max = 100): Promise<Analysis[]> {
  const snap = await getDocs(query(collection(db, 'analyses'), orderBy('updated_at', 'desc'), limit(max)))
  return snap.docs.map((d) => ({ id: d.id, ...d.data() }) as Analysis)
}
