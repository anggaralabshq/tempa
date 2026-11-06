import { useCallback, useEffect, useState } from "react";
import { type ContractAction, type ContractStatus, contractPreview, contractRun, contractStatus } from "../lib/ipc";

type Pending = { action: ContractAction; title: string; files: string[] };

const TITLES: Record<ContractAction, string> = {
  absorb: "Absorb direct edits into AGENTS.md and .tempa/, then sync. These files will be written:",
  discard: "Regenerate from the source and overwrite these files (direct edits are lost):",
  import: "Import the existing agent files into AGENTS.md, .tempa/ and tempa.toml:",
  init: "Create tempa.toml, AGENTS.md, HARNESS.md, CHANGES.md, docs/ and git hooks (existing files are never overwritten).",
  hooks: "Install the tempa git hooks (.tempa/hooks) and set core.hooksPath. Nothing else changes.",
};

function H({ children }: { children: string }) {
  return <h3 className="mt-4 mb-1 text-xs uppercase tracking-wide text-white/50">{children}</h3>;
}

/** Contract state of one project; `refresh` bumps when the watcher reports a change. */
export default function ContractPanel({ root, refresh = 0 }: { root: string | null; refresh?: number }) {
  const [s, setS] = useState<ContractStatus | null>(null);
  const [pending, setPending] = useState<Pending | null>(null);
  const [note, setNote] = useState<string | null>(null);

  const load = useCallback(() => {
    if (root) contractStatus(root).then(setS, (e) => setNote(String(e)));
  }, [root]);
  useEffect(() => {
    setS(null);
    setPending(null);
    setNote(null);
  }, [root]);
  useEffect(load, [load, refresh]);

  if (!root) return <p className="text-white/40">No project selected.</p>;
  if (!s) return <p className="text-white/40">Checking…</p>;

  const ask = async (action: ContractAction) => {
    setNote(null);
    try {
      const files = action === "absorb" || action === "discard" ? await contractPreview(root, action) : [];
      const absorbPreview = action === "import" ? await contractPreview(root, "absorb") : [];
      setPending({ action, title: TITLES[action], files: [...files, ...absorbPreview] });
    } catch (e) {
      setNote(String(e));
    }
  };
  const confirm = async () => {
    if (!pending) return;
    const { action } = pending;
    setPending(null);
    try {
      const changed = await contractRun(root, action);
      setNote(changed.length ? `${action}: ${changed.length} file(s) changed` : `${action}: nothing to change`);
    } catch (e) {
      setNote(String(e));
    }
    load();
  };

  const versionWarning =
    s.cli_version === null
      ? "tempa CLI not found on PATH: git hooks will fail. Install it (cargo install --path crates/tempa-cli)."
      : s.cli_version !== s.app_version
        ? `tempa CLI ${s.cli_version} differs from the app (${s.app_version}): hooks may generate different files. Reinstall the CLI.`
        : null;

  return (
    <div className="space-y-1 overflow-auto text-xs">
      <div className="flex items-center justify-between">
        <h2 className="text-xs uppercase tracking-wide text-white/50">Contract</h2>
        <button type="button" title="Refresh" onClick={load} className="text-white/50 hover:text-white">
          ↻
        </button>
      </div>
      {s.error && <p className="text-red-400">{s.error}</p>}
      {versionWarning && <p className="rounded bg-amber-500/10 p-2 text-amber-300">{versionWarning}</p>}

      {pending && (
        <div role="dialog" className="rounded border border-white/20 bg-white/5 p-2">
          <p>{pending.title}</p>
          {pending.files.length > 0 && (
            <ul className="my-2 max-h-40 overflow-auto font-mono text-white/70">
              {pending.files.map((f) => (
                <li key={f}>{f}</li>
              ))}
            </ul>
          )}
          <div className="mt-2 flex gap-2">
            <button type="button" onClick={confirm} className="rounded bg-sky-600 px-2 py-1 hover:bg-sky-500">
              Confirm
            </button>
            <button type="button" onClick={() => setPending(null)} className="rounded px-2 py-1 hover:bg-white/10">
              Cancel
            </button>
          </div>
        </div>
      )}
      {note && <p className="text-white/60">{note}</p>}

      {!s.has_config ? (
        <>
          <p className="text-white/60">This folder has no tempa.toml yet.</p>
          <button type="button" onClick={() => ask(s.offer ?? "init")} className="rounded bg-white/10 px-2 py-1 hover:bg-white/15">
            {s.offer === "import" ? "Import agent files" : "Set up tempa"}
          </button>
        </>
      ) : (
        <>
          <H>Documents</H>
          {s.findings.length === 0 ? (
            <p className="text-emerald-400">✓ Contract complete</p>
          ) : (
            <ul className="space-y-1">
              {s.findings.map((f) => (
                <li key={`${f.code}:${f.path}:${f.message}`}>
                  <span className={f.severity === "error" ? "text-red-400" : "text-amber-300"}>{f.severity === "error" ? "✗" : "!"}</span>{" "}
                  {f.path && <span className="font-mono">{f.path}</span>} <span className="text-white/60">{f.message}</span>
                </li>
              ))}
            </ul>
          )}

          <H>{`Drift (${s.drift.length})`}</H>
          {s.drift.length === 0 ? (
            <p className="text-emerald-400">✓ Every agent's files match the source</p>
          ) : (
            <>
              <ul className="space-y-1">
                {s.drift.map((f) => (
                  <li key={`${f.code}:${f.path}`}>
                    <span className="font-mono">{f.path}</span>
                    <span className="block text-white/60">{f.message}</span>
                  </li>
                ))}
              </ul>
              <div className="mt-1 flex gap-2">
                <button type="button" onClick={() => ask("absorb")} className="rounded bg-white/10 px-2 py-1 hover:bg-white/15">
                  Absorb
                </button>
                <button type="button" onClick={() => ask("discard")} className="rounded px-2 py-1 text-red-300 hover:bg-red-500/10">
                  Discard
                </button>
              </div>
            </>
          )}

          <H>Agents</H>
          <table className="w-full">
            <thead className="text-white/40">
              <tr>
                <th className="text-left font-normal">agent</th>
                <th className="text-right font-normal">files</th>
                <th className="text-right font-normal">gaps</th>
                <th className="text-right font-normal">tools</th>
              </tr>
            </thead>
            <tbody>
              {s.agents.map((a) => (
                <tr key={a.agent}>
                  <td>{a.agent}</td>
                  <td className="text-right">{a.files}</td>
                  <td className="text-right">{a.gaps}</td>
                  <td className="text-right">{a.lifecycle === "mcp" ? "✓" : "–"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}

      <H>Git hooks</H>
      {s.hooks_ok ? (
        <p className="text-emerald-400">✓ Installed</p>
      ) : (
        <button type="button" onClick={() => ask("hooks")} className="rounded bg-white/10 px-2 py-1 hover:bg-white/15">
          Install hooks
        </button>
      )}
    </div>
  );
}
