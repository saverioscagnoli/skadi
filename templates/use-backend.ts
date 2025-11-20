import { createContext, useContext } from "react";

type ExecFunction = (command: string) => Promise<string>;
type UseListenHook = (script: string, callback: (data: string) => void) => void;

type WindowHandle = {
  show: (label?: string) => Promise<void>;
  hide: (label?: string) => Promise<void>;
};

type BackendContextType = {
  exec: ExecFunction;
  useListen: UseListenHook;
  win: WindowHandle;
};

const BackendContext = createContext<BackendContextType | null>(null);

function useBackend() {
  const ctx = useContext(BackendContext);

  if (!ctx) {
    throw new Error("useBackend must be used within a BackendProvider");
  }

  return ctx;
}

export { BackendContext, useBackend };
export type { ExecFunction, UseListenHook, WindowHandle, BackendContextType };
