import { invoke } from "@tauri-apps/api/core";
import { useTauriEvent } from "@util-hooks/use-tauri-event";
import { usePlugins } from "~/hooks/use-plugin";

const App = () => {
  const { plugins, loading, error } = usePlugins();

  return (
    <div
      className="window"
      style={{
        width: "100vw",
        height: "100vh",
      }}
    >
      {plugins.map((Plugin) => (
        <Plugin.component
          invoke={invoke}
          useTauriEvent={useTauriEvent}
          key={Plugin.name}
        />
      ))}
    </div>
  );
};

export { App };
