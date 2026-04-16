import { useEffect, useRef } from "react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { HostConfig } from "../App";

interface Props {
  host: HostConfig;
  session: string;
}

export default function Terminal({ host, session }: Props) {
  const termRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const pidRef = useRef<number | null>(null);

  useEffect(() => {
    if (!termRef.current) return;

    const xterm = new XTerm({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: "SF Mono, Cascadia Code, Fira Code, JetBrains Mono, monospace",
      theme: {
        background: "#1a1b26",
        foreground: "#c0caf5",
        cursor: "#c0caf5",
        cursorAccent: "#1a1b26",
        selectionBackground: "#33467c",
        black: "#15161e",
        red: "#f7768e",
        green: "#9ece6a",
        yellow: "#e0af68",
        blue: "#7aa2f7",
        magenta: "#bb9af7",
        cyan: "#7dcfff",
        white: "#a9b1d6",
        brightBlack: "#414868",
        brightRed: "#f7768e",
        brightGreen: "#9ece6a",
        brightYellow: "#e0af68",
        brightBlue: "#7aa2f7",
        brightMagenta: "#bb9af7",
        brightCyan: "#7dcfff",
        brightWhite: "#c0caf5",
      },
    });

    const fit = new FitAddon();
    xterm.loadAddon(fit);
    xterm.loadAddon(new WebLinksAddon());

    xterm.open(termRef.current);
    fit.fit();

    xtermRef.current = xterm;
    fitRef.current = fit;

    const rows = xterm.rows;
    const cols = xterm.cols;

    invoke<number>("attach_tmux_session", {
      host: host.host,
      port: host.port,
      username: host.username,
      authType: host.auth_type,
      password: host.password || "",
      keyPath: host.key_path || "",
      sessionName: session,
      rows,
      cols,
    })
      .then((pid) => {
        pidRef.current = pid;
      })
      .catch((err) => {
        xterm.writeln(`\x1b[31mError attaching to session: ${err}\x1b[0m`);
      });

    const unlistenOutput = listen<string>("terminal-output", (event) => {
      xterm.write(event.payload);
    });

    const unlistenResize = listen<number>("terminal-resize", () => {
      fit.fit();
      invoke("resize_pty", {
        pid: pidRef.current,
        rows: xterm.rows,
        cols: xterm.cols,
      }).catch(() => {});
    });

    xterm.onData((data) => {
      invoke("write_to_pty", { pid: pidRef.current, data }).catch(() => {});
    });

    xterm.onResize(({ rows, cols }) => {
      invoke("resize_pty", { pid: pidRef.current, rows, cols }).catch(() => {});
    });

    const resizeObserver = new ResizeObserver(() => {
      fit.fit();
    });
    resizeObserver.observe(termRef.current);

    return () => {
      resizeObserver.disconnect();
      if (pidRef.current !== null) {
        invoke("kill_pty", { pid: pidRef.current }).catch(() => {});
      }
      unlistenOutput.then((f) => f());
      unlistenResize.then((f) => f());
      xterm.dispose();
      xtermRef.current = null;
      fitRef.current = null;
      pidRef.current = null;
    };
  }, [host, session]);

  return <div ref={termRef} style={{ width: "100%", height: "100%" }} />;
}
