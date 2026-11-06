import { useEffect, useReducer, useRef, useState } from "react";
import { type ChatStarted, chatCancel, chatPermission, chatPrompt, chatSetModel, chatStart, chatStop } from "../lib/ipc";
import { applyEvent, initial } from "./reducer";

type Props = { root: string; cwd: string; agent: string; task: string | null; focused: boolean };

/** One ACP agent session. Unmounting stops the agent. */
export default function Chat({ root, cwd, agent, task, focused }: Props) {
  const [s, dispatch] = useReducer(applyEvent, initial);
  const [started, setStarted] = useState<ChatStarted | null>(null);
  const [failed, setFailed] = useState<string | null>(null);
  const [model, setModel] = useState<string | null>(null);
  const [text, setText] = useState("");
  const input = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    let id: number | null = null;
    let gone = false;
    chatStart(root, cwd, agent, task, dispatch).then(
      (st) => {
        id = st.id;
        if (gone) chatStop(st.id);
        else {
          setStarted(st);
          setModel(st.current_model);
        }
      },
      (e) => setFailed(String(e)),
    );
    return () => {
      gone = true;
      if (id !== null) chatStop(id);
    };
  }, [root, cwd, agent, task]);

  useEffect(() => {
    if (focused) input.current?.focus();
  }, [focused]);

  const send = () => {
    const t = text.trim();
    if (!started || !t || s.busy) return;
    dispatch({ type: "user", text: t });
    setText("");
    chatPrompt(started.id, t).catch((e) => dispatch({ type: "error", message: String(e) }));
  };

  const answer = (request: number, option: string | null) => {
    if (!started) return;
    dispatch({ type: "answered", request });
    chatPermission(started.id, request, option);
  };

  if (failed) return <p className="whitespace-pre-wrap p-3 text-sm text-red-300">{failed}</p>;

  return (
    <div className="flex h-full flex-col bg-neutral-950 text-sm">
      <div className="flex h-7 shrink-0 items-center gap-2 border-b border-white/10 px-2 text-xs text-white/60">
        <span>{agent}</span>
        {task && <span className="text-sky-300/80">{task}</span>}
        {!started && <span>Starting… (first run downloads the adapter)</span>}
        {started && started.models.length > 0 && (
          <select
            aria-label="Model"
            value={model ?? ""}
            onChange={(e) => {
              const m = e.target.value;
              chatSetModel(started.id, m).then(
                () => setModel(m),
                (err) => dispatch({ type: "error", message: String(err) }),
              );
            }}
            className="ml-auto bg-transparent"
          >
            {started.models.map(([id, name]) => (
              <option key={id} value={id}>
                {name}
              </option>
            ))}
          </select>
        )}
      </div>
      <div className="min-h-0 flex-1 space-y-2 overflow-y-auto p-3">
        {s.plan.length > 0 && (
          <ul className="rounded border border-white/10 p-2 text-xs">
            {s.plan.map((p) => (
              <li key={p.content} className={p.status === "completed" ? "text-white/40 line-through" : ""}>
                {p.content}
              </li>
            ))}
          </ul>
        )}
        {s.items.map((it, i) => {
          const key = `${i}`;
          if (it.kind === "tool")
            return (
              <div key={it.id} className="rounded bg-white/5 px-2 py-1 font-mono text-xs text-white/70">
                {it.toolKind} · {it.title} · {it.status}
              </div>
            );
          if (it.kind === "note")
            return (
              <p key={key} className={`whitespace-pre-wrap text-xs ${it.tone === "error" ? "text-red-300" : "text-white/50"}`}>
                {it.text}
              </p>
            );
          return (
            <p key={key} className={`whitespace-pre-wrap ${it.kind === "user" ? "text-sky-200" : ""}`}>
              {it.text}
            </p>
          );
        })}
        {s.pending.map((p) => {
          const once = p.options.find((o) => o.kind === "allow_once");
          const reject = p.options.find((o) => o.kind === "reject_once");
          return (
            <div key={p.request} className="rounded border border-amber-400/40 p-2">
              <p className="text-xs text-white/60">The agent asks to run ({p.toolCall.kind ?? "tool"}):</p>
              <p className="font-mono">{p.toolCall.title}</p>
              {/* *_always options are never offered: every call is decided by the user or the policy. */}
              <div className="mt-2 flex gap-2">
                {once && (
                  <button type="button" onClick={() => answer(p.request, once.optionId)} className="rounded bg-white/10 px-2 py-1">
                    Allow once
                  </button>
                )}
                <button type="button" onClick={() => answer(p.request, reject?.optionId ?? null)} className="rounded px-2 py-1 text-white/70">
                  Reject
                </button>
              </div>
            </div>
          );
        })}
      </div>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          send();
        }}
        className="flex shrink-0 gap-2 border-t border-white/10 p-2"
      >
        <textarea
          ref={input}
          value={text}
          disabled={s.ended}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              send();
            }
          }}
          rows={2}
          placeholder={s.ended ? "The agent exited." : "Ask the agent…"}
          className="min-h-0 flex-1 resize-none rounded bg-white/5 p-2 outline-none"
        />
        {s.busy ? (
          <button type="button" onClick={() => started && chatCancel(started.id)} className="rounded bg-red-500/30 px-3">
            Stop
          </button>
        ) : (
          <button type="submit" disabled={!started || s.ended} className="rounded bg-white/10 px-3 disabled:opacity-40">
            Send
          </button>
        )}
      </form>
    </div>
  );
}
