import { useEffect, useRef, useState } from "react";

import type { Track } from "@/src/services/session/session.interfaces";

interface TrackNameProps {
  disabled: boolean;
  onRename: (id: string, name: string) => void;
  track: Track;
}

export function TrackName({ track, disabled, onRename }: TrackNameProps) {
  const [value, setValue] = useState(track.name);
  const cancelled = useRef(false);
  useEffect(() => setValue(track.name), [track.name]);
  return (
    <input
      className="track-name-input"
      aria-label={`Rename track ${track.name}`}
      title="Click to rename track"
      maxLength={80}
      disabled={disabled}
      value={value}
      onChange={(e) => setValue(e.target.value)}
      onBlur={() => {
        if (cancelled.current) {
          cancelled.current = false;
          setValue(track.name);
          return;
        }
        if (value.trim() && value.trim() !== track.name)
          onRename(track.id, value.trim());
        else setValue(track.name);
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter") e.currentTarget.blur();
        if (e.key === "Escape") {
          cancelled.current = true;
          e.currentTarget.blur();
        }
      }}
    />
  );
}
