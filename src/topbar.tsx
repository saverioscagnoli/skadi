import { createRoot } from "react-dom/client";
import plugins from "~/registry";
import { exec, useListen } from "~/util";

import "~/index.css";

const LABEL = "topbar";

createRoot(document.getElementById("root")!).render(
  <div className="w-screen h-screen topbar-window">
    {plugins.map(Plugin => (
      <Plugin key={Plugin.name} exec={exec(LABEL)} useListen={useListen} />
    ))}
  </div>
);