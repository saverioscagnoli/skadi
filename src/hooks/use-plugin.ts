import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as Babel from "@babel/standalone";
import React from "react";
import type { Plugin, PluginHookReturn } from "../types/plugin";

export function usePlugins(): PluginHookReturn {
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | undefined>();

  const compileJSXComponent = useCallback(
    (jsxCode: string, filename: string): React.ComponentType => {
      try {
        const isTypeScript =
          filename.endsWith(".tsx") || filename.endsWith(".ts");

        console.log(`Compiling plugin ${filename}`);

        const result = Babel.transform(jsxCode, {
          filename,
          presets: [
            ["react", { runtime: "classic" }],
            ...(isTypeScript
              ? [["typescript", { isTSX: true, allExtensions: true }]]
              : []),
          ],
          plugins: [["transform-react-jsx", { pragma: "React.createElement" }]],
        });

        if (!result.code) {
          throw new Error("Babel transformation returned empty code");
        }

        console.log(`Transformed code for ${filename}:`, result.code);

        // Create the component in a more direct way
        try {
          // Create a module-like environment
          const moduleExports = {};
          const module = { exports: moduleExports };

          // Create the function with proper scope
          const componentFunction = new Function(
            "React",
            "useState",
            "useEffect",
            "useCallback",
            "useMemo",
            "useRef",
            "module",
            "exports",
            `
            ${result.code}
            
            // Return the component - check multiple possible exports
            if (typeof Component !== 'undefined') {
              return Component;
            } else if (typeof exports.default !== 'undefined') {
              return exports.default;
            } else if (typeof module.exports !== 'undefined' && typeof module.exports === 'function') {
              return module.exports;
            } else {
              console.warn('No component found in plugin ${filename}');
              return function FallbackComponent() {
                return React.createElement('div', { 
                  style: { color: 'yellow', fontSize: '12px', padding: '4px' } 
                }, 'No component exported from ${filename}');
              };
            }
          `
          );

          const component = componentFunction(
            React,
            React.useState,
            React.useEffect,
            React.useCallback,
            React.useMemo,
            React.useRef,
            module,
            moduleExports
          );

          console.log(
            "Created component for",
            filename,
            ":",
            typeof component,
            component.name
          );

          // Validate that we got a function
          if (typeof component !== "function") {
            throw new Error(
              `Plugin ${filename} did not export a valid React component`
            );
          }

          return component as React.ComponentType;
        } catch (executionError) {
          console.error(`Error executing plugin ${filename}:`, executionError);
          return () =>
            React.createElement(
              "div",
              { style: { color: "orange", fontSize: "12px", padding: "4px" } },
              `Execution Error: ${filename}`
            );
        }
      } catch (compileError) {
        console.error(`Error compiling plugin ${filename}:`, compileError);
        return () =>
          React.createElement(
            "div",
            { style: { color: "red", fontSize: "12px", padding: "4px" } },
            `Compile Error: ${filename}`
          );
      }
    },
    []
  );

  const loadPlugins = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(undefined);

    try {
      const filenames = await invoke<string[]>("get_plugin_files");
      console.log("Found plugin files:", filenames);

      const loadedPlugins: Plugin[] = [];

      for (const filename of filenames) {
        try {
          const jsxCode = await invoke<string>("read_plugin_file", {
            filename,
          });
          console.log(
            `Loaded code for ${filename}:`,
            jsxCode.substring(0, 200)
          );

          const component = compileJSXComponent(jsxCode, filename);

          loadedPlugins.push({
            name: filename.replace(/\.(jsx|tsx)$/, ""),
            component,
            filename,
          });
        } catch (pluginError) {
          console.error(`Error loading plugin ${filename}:`, pluginError);
          loadedPlugins.push({
            name: filename.replace(/\.(jsx|tsx)$/, ""),
            component: () =>
              React.createElement(
                "div",
                { style: { color: "red", fontSize: "12px" } },
                `Failed: ${filename}`
              ),
            filename,
            error:
              pluginError instanceof Error
                ? pluginError.message
                : String(pluginError),
          });
        }
      }

      console.log("Loaded plugins:", loadedPlugins);
      setPlugins(loadedPlugins);
    } catch (loadError) {
      const errorMessage =
        loadError instanceof Error ? loadError.message : String(loadError);
      console.error("Error loading plugins:", loadError);
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [compileJSXComponent]);

  useEffect(() => {
    loadPlugins();
  }, [loadPlugins]);

  return {
    plugins,
    loading,
    reloadPlugins: loadPlugins,
    error,
  };
}
