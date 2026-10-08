import type { Firestore } from "firebase-admin/firestore";
import { ANALYSES } from "./mcp.js";
import { NODE_LABELS } from "./nodeLabels.js";

/** Chain node as served to the viewer: [id, label, role ("C" | "F" | ""), proposed (0 | 1), children?]. */
export type ChainNode = [string, string, string, 0 | 1, ChainNode[]?];

/** One involvement as served by GET /api/chains. Short keys keep the payload small. */
export interface ChainRecord {
  e: string; // ev_id
  k: number; // aircraft_key
  n: string; // ntsb_no
  d: string; // event_date
  o: string; // outcome
  f: number | null; // fatalities
  l: string; // location
  m: string; // make + model
  md: string; // model
  c: string; // aircraft class
  p: string; // operation
  w: string; // conditions
  h: string; // headline (first line of the summary)
  pc: string; // NTSB probable cause
  t: ChainNode; // narrative chain
}

interface RawNode {
  node?: string;
  label?: string;
  role?: string | null;
  whys?: RawNode[];
}

const ID = /\b([a-z]+(?:\.[a-z0-9_]+)+)/;
const title = (s: string) => s.toLowerCase().replace(/\b\w/g, (c) => c.toUpperCase());

/** Normalise a stored narrative-chain node: agents wrote ids like "proposed: latent.pilot.distraction". */
export function toChainNode(raw: RawNode): ChainNode {
  const text = raw.node ?? "";
  const id = text.match(ID)?.[1] ?? (text || "?");
  const proposed = /propos/i.test(text) || !(id in NODE_LABELS);
  let label = raw.label?.trim() || NODE_LABELS[id] || id;
  if (proposed && label.toLowerCase() === id) label = id.split(".").at(-1)!.replace(/_/g, " ");
  label = label.charAt(0).toUpperCase() + label.slice(1);
  const role = raw.role === "C" || raw.role === "F" ? raw.role : "";
  const kids = (raw.whys ?? []).map(toChainNode);
  return kids.length ? [id, label, role, proposed ? 1 : 0, kids] : [id, label, role, proposed ? 1 : 0];
}

/** Read every analysis and build the viewer payload. */
export async function buildChains(db: Firestore) {
  const snap = await db
    .collection(ANALYSES)
    .select("ev_id", "aircraft_key", "ntsb_no", "event_date", "outcome", "aircraft", "summary", "event", "method_version", "analysis.narrative_chain")
    .get();
  const records: ChainRecord[] = [];
  const methods = new Set<string>();
  for (const doc of snap.docs) {
    const a = doc.data();
    const tree = a.analysis?.narrative_chain?.tree as RawNode | undefined;
    if (!tree) continue;
    const ac = a.aircraft ?? {};
    const ev = a.event ?? {};
    methods.add(a.method_version ?? "");
    records.push({
      e: a.ev_id,
      k: a.aircraft_key ?? 1,
      n: a.ntsb_no ?? "",
      d: a.event_date ?? "",
      o: a.outcome ?? "unknown",
      f: typeof ev.fatalities === "number" ? ev.fatalities : null,
      l: ev.location ?? "",
      m: [ac.make ? title(ac.make) : "", ac.model ?? ""].filter(Boolean).join(" "),
      md: ac.model ?? "",
      c: ac.class_id ?? "",
      p: ev.operation ?? "",
      w: ev.conditions ?? "",
      h: String(a.summary ?? "").split("\n")[0].replace(/\*+/g, "").trim(),
      pc: ev.probable_cause ?? "",
      t: toChainNode(tree),
    });
  }
  records.sort((x, y) => (x.d < y.d ? 1 : x.d > y.d ? -1 : x.e < y.e ? 1 : -1));
  return { generated: new Date().toISOString(), methods: [...methods].filter(Boolean).sort(), labels: NODE_LABELS, records };
}
