import React, { useEffect, useState } from "react";
import { cn } from "../util";
import { Props } from "../types";

type WorkspaceInfo = {
  active: number;
  total: number;
};

const Workspaces: React.FC<Props> = ({ exec, useListen }) => {
  const [workspaceInfo, setWorkspaceInfo] = useState<WorkspaceInfo>({
    active: 1,
    total: 1
  });

  useEffect(() => {
    exec({
      script: "/home/svscagn/.config/skadi/scripts/workspace.sh",
      polls: true
    });
  }, []);

  useListen<WorkspaceInfo>(
    "/home/svscagn/.config/skadi/scripts/workspace.sh",
    data => {
      data.total = Math.max(data.active, data.total);
      console.log("Workspace data received:", data);
      setWorkspaceInfo(data);
    }
  );

  return (
    <div
      className={cn(
        "flex items-center gap-2",
        "rounded-lg",
        "bg-black/50 text-white",
        "border-2 border-white/10",
        "px-2 py-1.5"
      )}
    >
      <div className="flex gap-1">
        {Array.from({ length: workspaceInfo.total }, (_, i) => {
          const workspaceNum = i + 1;
          const isActive = workspaceNum === workspaceInfo.active;

          return (
            <div
              key={workspaceNum}
              className={cn(
                "px-2 py-0.5 rounded text-sm font-medium",
                "transition-all duration-300 ease-out",
                isActive
                  ? "bg-white/20 text-white scale-105"
                  : "text-white/50 hover:text-white/70"
              )}
            >
              {workspaceNum}
            </div>
          );
        })}
      </div>
    </div>
  );
};

export { Workspaces };
