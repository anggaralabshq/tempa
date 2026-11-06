import { useEffect, useState } from "react";
import { type Agent, agentList } from "../lib/ipc";

/** The five agents; a missing one is disabled with its install hint. */
export default function AgentPicker({ onPick }: { onPick: (id: string) => void }) {
  const [agents, setAgents] = useState<Agent[] | null>(null);
  useEffect(() => {
    agentList().then(setAgents, () => setAgents([]));
  }, []);
  if (!agents) return <p className="p-3 text-xs text-white/50">Looking for agents…</p>;
  return (
    <div className="flex flex-col gap-1 p-2 text-sm">
      {agents.map((a) => (
        <button
          key={a.id}
          type="button"
          disabled={!a.installed}
          title={a.hint ?? undefined}
          onClick={() => onPick(a.id)}
          className="flex items-center justify-between rounded px-3 py-2 text-left hover:bg-white/10 disabled:opacity-40"
        >
          {a.name}
          {!a.lifecycle && <span className="text-xs text-amber-300/80">context-only</span>}
        </button>
      ))}
    </div>
  );
}
