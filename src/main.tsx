import { createRoot } from "react-dom/client";
import plugins from "~/registry";

import "~/index.css";

createRoot(document.getElementById("root")!).render(
  <div className="w-screen h-screen main-window">
    <h1>main Window</h1>
    {plugins.map(Plugin => (
      <Plugin key={Plugin.name} />
    ))}
  </div>
);