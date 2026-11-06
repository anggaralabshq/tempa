import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Terminal as XTerm } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import { useEffect, useRef } from "react";
import { ptyKill, ptyResize, ptySpawn, ptyWrite } from "../lib/ipc";

type Props = { cwd: string | null; focused?: boolean; onExit?: (code: number) => void };

/** One xterm bound to one pty. Unmounting kills the shell. */
export default function Terminal({ cwd, focused = true, onExit }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const termRef = useRef<XTerm | null>(null);
  const exitRef = useRef(onExit);
  exitRef.current = onExit;

  useEffect(() => {
    const el = host.current;
    if (!el) return;
    const term = new XTerm({
      fontFamily: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
      fontSize: 13,
      cursorBlink: true,
      theme: { background: "#000000" },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.loadAddon(new WebLinksAddon());
    term.open(el);
    termRef.current = term;
    fit.fit();

    let id: number | null = null;
    let disposed = false;
    ptySpawn(cwd, term.cols, term.rows, {
      onData: (b) => !disposed && term.write(b),
      onExit: (code) => !disposed && exitRef.current?.(code),
    }).then(
      (pty) => {
        if (disposed) void ptyKill(pty);
        else id = pty;
      },
      (e) => term.write(`\r\n[tempa] could not start a shell: ${e}\r\n`),
    );
    const input = term.onData((d) => id !== null && void ptyWrite(id, d));
    const resize = term.onResize(({ cols, rows }) => id !== null && void ptyResize(id, cols, rows));
    // Hidden tabs have no size; fit when they are shown again.
    const observer = new ResizeObserver(() => el.clientWidth > 0 && fit.fit());
    observer.observe(el);
    return () => {
      termRef.current = null;
      disposed = true;
      observer.disconnect();
      input.dispose();
      resize.dispose();
      if (id !== null) void ptyKill(id);
      term.dispose();
    };
  }, [cwd]);

  useEffect(() => {
    if (focused) termRef.current?.focus();
  }, [focused]);

  return <div ref={host} className="h-full w-full p-1" />;
}
