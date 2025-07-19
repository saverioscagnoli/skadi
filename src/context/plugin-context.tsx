import { createContext, useContext, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import type { PluginAPI } from "../types/plugin";

const PluginContext = createContext<PluginAPI | null>(null);

interface PluginProviderProps {
  children: ReactNode;
}

export function PluginProvider({ children }: PluginProviderProps): JSX.Element {
  const pluginAPI: PluginAPI = {
    invoke: async <T = any,>(
      cmd: string,
      args?: Record<string, any>
    ): Promise<T> => {
      return invoke<T>(cmd, args);
    },

    openExternal: async (url: string): Promise<void> => {
      // Add URL validation
      try {
        new URL(url); // This will throw if URL is invalid
        await open(url);
      } catch (error) {
        throw new Error(`Invalid URL: ${url}`);
      }
    },
  };

  return (
    <PluginContext.Provider value={pluginAPI}>
      {children}
    </PluginContext.Provider>
  );
}

export const usePluginAPI = (): PluginAPI => {
  const context = useContext(PluginContext);
  if (!context) {
    throw new Error("usePluginAPI must be used within a PluginProvider");
  }
  return context;
};
