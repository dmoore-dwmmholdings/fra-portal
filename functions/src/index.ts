import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import { initializeApp } from "firebase-admin/app";
import { getFirestore } from "firebase-admin/firestore";
import { defineSecret } from "firebase-functions/params";
import { onRequest } from "firebase-functions/v2/https";
import { createHash, timingSafeEqual } from "node:crypto";
import { brotliCompressSync, gzipSync } from "node:zlib";
import { buildChains } from "./chains.js";
import { buildServer } from "./mcp.js";

initializeApp();

/** Bearer token Claude Code sends. Set with `firebase functions:secrets:set MCP_TOKEN`. */
const MCP_TOKEN = defineSecret("MCP_TOKEN");

const digest = (s: string) => createHash("sha256").update(s).digest();

function authorized(header: string | undefined): boolean {
  const token = header?.match(/^Bearer (.+)$/)?.[1];
  return !!token && timingSafeEqual(digest(token), digest(MCP_TOKEN.value()));
}

// Stateless Streamable HTTP: a fresh server + transport per request, JSON responses (no SSE),
// so it works behind the Hosting rewrite and needs no session affinity. Cloud Run access is public;
// the bearer token is the access check.
export const mcp = onRequest(
  { secrets: [MCP_TOKEN], region: "us-central1", invoker: "public", maxInstances: 3, timeoutSeconds: 60 },
  async (req, res) => {
    if (!authorized(req.get("authorization"))) {
      res.status(401).json({ error: "unauthorized" });
      return;
    }
    if (req.method !== "POST") {
      res.status(405).set("Allow", "POST").json({ error: "method not allowed" });
      return;
    }
    const server = buildServer();
    const transport = new StreamableHTTPServerTransport({
      sessionIdGenerator: undefined,
      enableJsonResponse: true,
    });
    res.on("close", () => {
      void transport.close();
      void server.close();
    });
    await server.connect(transport);
    await transport.handleRequest(req, res, req.body);
  },
);

// Public, read-only payload for the chain viewer: every analysis, compacted. The Hosting CDN caches it,
// so a new upload shows within about five minutes.
export const chains = onRequest(
  { region: "us-central1", invoker: "public", maxInstances: 3, timeoutSeconds: 30 },
  async (req, res) => {
    if (req.method !== "GET") {
      res.status(405).set("Allow", "GET").json({ error: "method not allowed" });
      return;
    }
    // Hosting does not compress function responses, so compress here (120 KB of JSON -> ~20 KB).
    const body = await chainsBody();
    const accept = req.get("accept-encoding") ?? "";
    const [encoding, payload] = accept.includes("br") ? ["br", body.br] : accept.includes("gzip") ? ["gzip", body.gzip] : [null, body.raw];
    res.set({ "Cache-Control": "public, max-age=60, s-maxage=300", "Content-Type": "application/json; charset=utf-8", Vary: "Accept-Encoding" });
    if (encoding) res.set("Content-Encoding", encoding);
    res.send(payload);
  },
);

let cached: { at: number; raw: Buffer; br: Buffer; gzip: Buffer } | null = null;

/** Built payload, kept for 60 s per instance so repeated CDN misses do not re-read Firestore. */
async function chainsBody() {
  if (cached && Date.now() - cached.at < 60_000) return cached;
  const raw = Buffer.from(JSON.stringify(await buildChains(getFirestore())));
  cached = { at: Date.now(), raw, br: brotliCompressSync(raw), gzip: gzipSync(raw) };
  return cached;
}
