// Fake ACP agent for tests: newline-delimited JSON-RPC on stdio.
// Prompts: "echo <x>", "mcp", "edit <path>", "hang", "crash", "spawn-child".
// Env: FAKE_SILENT=1 never answers; FAKE_AUTH=1 fails session/new with auth_required.
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";

const rl = createInterface({ input: process.stdin });
const waiting = new Map();
let nextId = 1000;
let mcpServers = null;
let cancelTurn = null;

const send = (m) => process.stdout.write(`${JSON.stringify({ jsonrpc: "2.0", ...m })}\n`);
const update = (u) => send({ method: "session/update", params: { sessionId: "s1", update: u } });
const say = (text) => update({ sessionUpdate: "agent_message_chunk", content: { type: "text", text } });
const ask = (method, params) =>
  new Promise((resolve) => {
    const id = nextId++;
    waiting.set(id, resolve);
    send({ id, method, params });
  });

rl.on("line", async (line) => {
  if (process.env.FAKE_SILENT) return;
  const m = JSON.parse(line);
  if (m.id !== undefined && !m.method) {
    const resolve = waiting.get(m.id);
    waiting.delete(m.id);
    return resolve?.(m.result ?? m.error);
  }
  const reply = (result) => send({ id: m.id, result });
  switch (m.method) {
    case "initialize":
      return reply({ protocolVersion: 1, agentCapabilities: { loadSession: false }, authMethods: [] });
    case "session/new":
      if (process.env.FAKE_AUTH) {
        return send({ id: m.id, error: { code: -32000, message: "Authentication required: run fake login" } });
      }
      mcpServers = m.params.mcpServers;
      return reply({
        sessionId: "s1",
        configOptions: [{ id: "model", name: "Model", category: "model", type: "select", currentValue: "fast",
          options: [{ value: "fast", name: "Fast" }, { value: "smart", name: "Smart" }] }],
      });
    case "session/set_config_option":
      return reply({ configOptions: [] });
    case "session/cancel":
      return cancelTurn?.();
    case "session/prompt":
      return prompt(m);
    default:
      if (m.id !== undefined) send({ id: m.id, error: { code: -32601, message: "method not found" } });
  }
});

async function prompt(m) {
  const text = m.params.prompt.map((b) => b.text).join("");
  const [cmd, ...rest] = text.split(" ");
  const arg = rest.join(" ");
  const end = (stopReason = "end_turn") => send({ id: m.id, result: { stopReason } });
  switch (cmd) {
    case "echo":
      say(`echo ${arg}`);
      update({ sessionUpdate: "plan", entries: [{ content: "step", priority: "medium", status: "pending" }] });
      return end();
    case "mcp":
      say(JSON.stringify(mcpServers));
      return end();
    case "edit": {
      const r = await ask("session/request_permission", {
        sessionId: "s1",
        toolCall: { toolCallId: "t1", title: `edit ${arg}`, kind: "edit", locations: [{ path: arg }] },
        options: [
          { optionId: "a1", name: "Allow", kind: "allow_once" },
          { optionId: "a2", name: "Always", kind: "allow_always" },
          { optionId: "r1", name: "Reject", kind: "reject_once" },
        ],
      });
      say(JSON.stringify(r.outcome));
      return end();
    }
    case "hang":
      cancelTurn = () => {
        cancelTurn = null;
        end("cancelled");
      };
      return;
    case "crash":
      process.stderr.write("fake agent crashed\n", () => process.exit(3));
      return;
    case "spawn-child": {
      const c = spawn(process.execPath, ["-e", "setInterval(()=>{},1000)"], { stdio: "ignore" });
      say(String(c.pid));
      return end();
    }
    default:
      say(`unknown ${cmd}`);
      return end();
  }
}
