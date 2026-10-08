import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { FieldValue, getFirestore, type Firestore } from "firebase-admin/firestore";
import { z } from "zod";

/** Firestore collection holding one document per aircraft involvement. */
export const ANALYSES = "analyses";

export const docId = (evId: string, aircraftKey: number) => `${evId}_${aircraftKey}`;

const key = {
  // Pre-2021 ids look like 20080107X00026; CAROL-era ids are 14 digits, e.g. 20250510200140.
  ev_id: z.string().regex(/^(\d{8}X\d{5}|\d{14})$/).describe("NTSB event id, e.g. 20080107X00026 or 20250510200140"),
  aircraft_key: z.number().int().min(1).describe("NTSB Aircraft_Key (1 unless multi-aircraft event)"),
};

/** Event facts the chain viewer shows; taken from the NTSB record, not the analysis. */
const event = z
  .object({
    location: z.string().optional().describe("City, ST"),
    operation: z.string().optional().describe("e.g. Part 91, Part 135, Public use"),
    conditions: z.string().optional().describe("e.g. VMC, daylight"),
    fatalities: z.number().int().min(0).optional(),
    probable_cause: z.string().optional().describe("NTSB probable cause (narr_cause), verbatim"),
  })
  .optional();

const aircraft = z
  .object({
    make: z.string().optional(),
    model: z.string().optional(),
    class_id: z.string().optional(),
    family_id: z.string().optional(),
  })
  .optional();

const json = (v: unknown) => ({ content: [{ type: "text" as const, text: JSON.stringify(v, null, 2) }] });

export function buildServer(db: Firestore = getFirestore()): McpServer {
  const server = new McpServer({ name: "fra-portal", version: "0.1.0" });
  const col = db.collection(ANALYSES);

  server.registerTool(
    "put_analysis",
    {
      title: "Upload analysis",
      description:
        "Create or replace the analysis for one aircraft involvement (ev_id + aircraft_key). " +
        "Indexed fields support filtering in the portal; `analysis` is free-form JSON (chain, findings, notes).",
      inputSchema: {
        ...key,
        ntsb_no: z.string().optional(),
        event_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/).optional(),
        outcome: z.enum(["fatal", "serious", "minor", "none", "unknown"]).optional(),
        aircraft,
        event,
        summary: z.string().min(1).describe("Plain-language summary of the chain, markdown allowed"),
        nodes: z.array(z.string()).optional().describe("Causal node ids on the chain, for filtering"),
        analysis: z.record(z.string(), z.unknown()).describe("Free-form structured analysis"),
        method_version: z.string().default("wwca-0.1"),
        analyst: z.string().optional().describe("Model or person that produced it"),
      },
    },
    async (args) => {
      const id = docId(args.ev_id, args.aircraft_key);
      const ref = col.doc(id);
      const prev = await ref.get();
      await ref.set({
        ...args,
        updated_at: FieldValue.serverTimestamp(),
        created_at: prev.exists ? prev.get("created_at") : FieldValue.serverTimestamp(),
      });
      return json({ id, status: prev.exists ? "replaced" : "created" });
    },
  );

  server.registerTool(
    "patch_analysis",
    {
      title: "Patch analysis fields",
      description:
        "Merge indexed fields into an existing analysis without resending `analysis`. " +
        "`event` and `aircraft` merge key by key; other fields replace.",
      inputSchema: {
        ...key,
        ntsb_no: z.string().optional(),
        event_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/).optional(),
        outcome: z.enum(["fatal", "serious", "minor", "none", "unknown"]).optional(),
        aircraft,
        event,
        summary: z.string().min(1).optional(),
        nodes: z.array(z.string()).optional(),
      },
    },
    async ({ ev_id, aircraft_key, ...fields }) => {
      const ref = col.doc(docId(ev_id, aircraft_key));
      if (!(await ref.get()).exists) return json({ error: "not found" });
      // set+merge merges nested maps (event, aircraft) instead of replacing them
      await ref.set({ ...fields, updated_at: FieldValue.serverTimestamp() }, { merge: true });
      return json({ id: ref.id, status: "patched", fields: Object.keys(fields) });
    },
  );

  server.registerTool(
    "get_analysis",
    {
      title: "Get analysis",
      description: "Read the stored analysis for one aircraft involvement.",
      inputSchema: key,
      annotations: { readOnlyHint: true },
    },
    async ({ ev_id, aircraft_key }) => {
      const snap = await col.doc(docId(ev_id, aircraft_key)).get();
      return snap.exists ? json({ id: snap.id, ...snap.data() }) : json({ error: "not found" });
    },
  );

  server.registerTool(
    "list_analyses",
    {
      title: "List analyses",
      description: "List stored analyses (ids and summaries), newest first. Use to check coverage before analyzing.",
      inputSchema: {
        ev_id: z.string().optional().describe("Only this event"),
        limit: z.number().int().min(1).max(500).default(50),
        start_after: z.string().optional().describe("Document id from a previous page"),
      },
      annotations: { readOnlyHint: true },
    },
    async ({ ev_id, limit, start_after }) => {
      let q = ev_id ? col.where("ev_id", "==", ev_id) : col.orderBy("updated_at", "desc");
      if (start_after && !ev_id) {
        const cursor = await col.doc(start_after).get();
        if (cursor.exists) q = q.startAfter(cursor);
      }
      const snap = await q.limit(limit).select("ev_id", "aircraft_key", "summary", "updated_at").get();
      const rows = snap.docs.map((d) => ({ id: d.id, ...d.data() }));
      return json({ count: rows.length, next: rows.length === limit ? rows.at(-1)?.id : null, rows });
    },
  );

  return server;
}
