import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import { initializeApp } from "firebase-admin/app";
import { defineSecret } from "firebase-functions/params";
import { onRequest } from "firebase-functions/v2/https";
import { createHash, timingSafeEqual } from "node:crypto";
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
// so it works behind the Hosting rewrite and needs no session affinity.
export const mcp = onRequest(
  { secrets: [MCP_TOKEN], region: "us-central1", maxInstances: 3, timeoutSeconds: 60 },
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
