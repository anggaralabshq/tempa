export default function App() {
  return (
    <div className="flex h-full text-sm">
      <nav aria-label="Projects" className="w-56 shrink-0 border-r border-white/10 p-3">
        <h2 className="mb-2 text-xs uppercase tracking-wide text-white/50">Projects</h2>
      </nav>
      <main aria-label="Terminal" className="min-w-0 flex-1 bg-black" />
      <aside aria-label="Contract" className="w-72 shrink-0 border-l border-white/10 p-3">
        <h2 className="mb-2 text-xs uppercase tracking-wide text-white/50">Contract</h2>
      </aside>
    </div>
  );
}
