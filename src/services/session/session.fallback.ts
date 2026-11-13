import type { SessionState } from "@/src/services/session/session.interfaces";

export const emptySession: SessionState = {
  sampleRate: 0,
  storagePath: "",
  tracks: [],
};
