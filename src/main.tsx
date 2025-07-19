import ReactDOM from "react-dom/client";
import App from "./App";
import { PluginProvider } from "./context/plugin-context";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <PluginProvider>
    <App />
  </PluginProvider>
);
