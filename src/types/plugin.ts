import React, { ComponentType } from "react";

type PluginComponentProps = {
  invoke: typeof import("@tauri-apps/api/core").invoke;
  useTauriEvent: typeof import("@util-hooks/use-tauri-event").useTauriEvent;
};

export interface Plugin {
  name: string;
  component: React.ComponentType<PluginComponentProps>;
  filename: string;
  error?: string;
}

export interface PluginAPI {
  invoke: <T = any>(cmd: string, args?: Record<string, any>) => Promise<T>;
  openExternal?: (url: string) => Promise<void>;
}

export interface PluginHookReturn {
  plugins: Plugin[];
  loading: boolean;
  reloadPlugins: () => Promise<void>;
  error?: string;
}
