import type { Project } from "../lib/ipc";

type Props = {
  projects: Project[];
  active: string | null;
  onSelect: (id: string) => void;
  onAdd: () => void;
  onRemove: (id: string) => void;
};

export default function Sidebar({ projects, active, onSelect, onAdd, onRemove }: Props) {
  return (
    <div className="flex h-full flex-col">
      <h2 className="mb-2 text-xs uppercase tracking-wide text-white/50">Projects</h2>
      <ul className="flex-1 space-y-0.5 overflow-auto">
        {projects.map((p) => (
          <li key={p.id} className={`group flex items-center rounded ${p.id === active ? "bg-white/10" : "hover:bg-white/5"}`}>
            <button type="button" title={p.path} onClick={() => onSelect(p.id)} className="flex-1 truncate px-2 py-1 text-left">
              {p.name}
            </button>
            <button
              type="button"
              aria-label={`Remove ${p.name}`}
              title="Remove from tempa (closes its terminals; the folder is not touched)"
              onClick={() => onRemove(p.id)}
              className="px-2 text-white/30 opacity-0 hover:text-white group-hover:opacity-100"
            >
              ×
            </button>
          </li>
        ))}
      </ul>
      <button type="button" onClick={onAdd} className="mt-2 rounded px-2 py-1 text-left text-white/60 hover:bg-white/5 hover:text-white">
        + Add project
      </button>
    </div>
  );
}
