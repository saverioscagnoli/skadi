import { createContext, useContext } from "react";

type BackendMessage = {
  success: boolean;
  stdout: string;
  stderr: string;
};

type ExecFunction = (
  command: string,
  args: string[],
) => Promise<BackendMessage>;

type UseListenHook = <T = string>(
  script: string,
  args: string[],
  callback: (data: T) => void,
) => void;

type WindowHandle = {
  show: () => Promise<void>;
};

type BackendContextType = {
  exec: ExecFunction;
  useListen: UseListenHook;
  window: WindowHandle;
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
export type {
  BackendMessage,
  ExecFunction,
  UseListenHook,
  WindowHandle,
  BackendContextType,
};
