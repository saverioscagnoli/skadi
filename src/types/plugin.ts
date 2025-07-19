import { ComponentType } from "react";

export interface Plugin {
  name: string;
  component: ComponentType;
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
