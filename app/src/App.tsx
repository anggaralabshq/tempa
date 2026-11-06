import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import ContractPanel from "./contract/ContractPanel";
import Workspace from "./layout/Workspace";
import { type Project, projectsAdd, projectsList, projectsRemove } from "./lib/ipc";
import Sidebar from "./sidebar/Sidebar";

export default function App() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [active, setActive] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    projectsList().then((ps) => {
      setProjects(ps);
      setActive(ps[0]?.id ?? null);
    }, (e) => setError(String(e)));
  }, []);

  const add = async () => {
    const path = await open({ directory: true, title: "Add project" });
    if (typeof path !== "string") return;
    try {
      const p = await projectsAdd(path);
      setProjects((ps) => (ps.some((x) => x.id === p.id) ? ps : [...ps, p]));
      setActive(p.id);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const remove = async (id: string) => {
    await projectsRemove(id);
    const rest = projects.filter((p) => p.id !== id);
    setProjects(rest);
    if (active === id) setActive(rest[0]?.id ?? null);
  };

  return (
    <div className="flex h-full text-sm">
      <nav aria-label="Projects" className="w-56 shrink-0 border-r border-white/10 p-3">
        <Sidebar projects={projects} active={active} onSelect={setActive} onAdd={add} onRemove={remove} />
        {error && <p className="mt-2 text-xs text-red-400">{error}</p>}
      </nav>
      <main aria-label="Terminal" className="relative min-w-0 flex-1 bg-black">
        {projects.length === 0 && (
          <button type="button" onClick={add} className="absolute inset-0 m-auto h-9 w-44 rounded bg-white/10 hover:bg-white/15">
            Add a project folder
          </button>
        )}
        {/* Every project stays mounted so switching never kills its terminals. */}
        {projects.map((p) => (
          <div key={p.id} className="absolute inset-0" style={{ display: p.id === active ? "block" : "none" }}>
            <Workspace cwd={p.path} active={p.id === active} />
          </div>
        ))}
      </main>
      <aside aria-label="Contract" className="w-72 shrink-0 overflow-auto border-l border-white/10 p-3">
        <ContractPanel root={projects.find((p) => p.id === active)?.path ?? null} />
      </aside>
    </div>
  );
}
