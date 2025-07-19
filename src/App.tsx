import { usePlugins } from "./hooks/use-plugin";
import { Plugin } from "./types/plugin";
import { invoke } from "@tauri-apps/api/core";
import "./App.css"; // Assuming you have some styles for the app
import { useTauriEvent } from "@util-hooks/use-tauri-event";

function App({
  className = "",
  title = "Skadi",
}: {
  className?: string;
  title?: string;
}): JSX.Element {
  const { plugins, loading, error } = usePlugins();

  return (
    <div className={`title-bar ${className}`} data-tauri-drag-region>
      <div className="title-bar-content">
        <div className="plugin-area">
          {plugins.map((plugin: Plugin) => {
            console.log(plugin);
            const PluginComponent = plugin.component;
            return (
              <div
                key={plugin.name}
                className="plugin-wrapper"
                data-plugin={plugin.name}
              >
                <PluginComponent
                  invoke={invoke}
                  useTauriEvent={useTauriEvent}
                />
              </div>
            );
          })}
        </div>

        <div className="window-controls">{/* Your window controls here */}</div>
      </div>
    </div>
  );
}

export default App;
