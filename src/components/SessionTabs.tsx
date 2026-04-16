import { useState } from "react";
import type { TmuxSession } from "../App";

interface Props {
  sessions: TmuxSession[];
  activeSession: string | null;
  onSelect: (name: string) => void;
  onCreate: (name: string) => void;
  onKill: (name: string) => void;
}

export default function SessionTabs({ sessions, activeSession, onSelect, onCreate, onKill }: Props) {
  const [showNew, setShowNew] = useState(false);
  const [newName, setNewName] = useState("");

  const handleCreate = () => {
    if (!newName.trim()) return;
    onCreate(newName.trim());
    setNewName("");
    setShowNew(false);
  };

  return (
    <div className="session-tabs">
      {sessions.map((s) => (
        <div
          key={s.name}
          className={`session-tab ${activeSession === s.name ? "active" : ""}`}
          onClick={() => onSelect(s.name)}
        >
          {s.name}
          <span
            className="session-tab-close"
            onClick={(e) => {
              e.stopPropagation();
              onKill(s.name);
            }}
          >
            ✕
          </span>
        </div>
      ))}

      {showNew ? (
        <input
          autoFocus
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          onBlur={() => { setShowNew(false); setNewName(""); }}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleCreate();
            if (e.key === "Escape") { setShowNew(false); setNewName(""); }
          }}
          placeholder="session name"
          style={{
            background: "#16161e",
            border: "1px solid #7aa2f7",
            borderRadius: "4px",
            color: "#c0caf5",
            padding: "4px 8px",
            fontSize: "13px",
            width: "120px",
            outline: "none",
            fontFamily: "inherit",
          }}
        />
      ) : (
        <button className="btn-new-session" onClick={() => setShowNew(true)}>
          + New Session
        </button>
      )}
    </div>
  );
}
