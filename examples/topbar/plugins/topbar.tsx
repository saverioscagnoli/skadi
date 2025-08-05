import React, { use } from "react";
import { Props } from "./types";
import { cn } from "./util";
import { Clock } from "./components/clock";
import { SysInfo } from "./components/sysinfo";
import { Workspaces } from "./components/workspaces";
import { Spotify } from "./components/player";

const Topbar: React.FC<Props> = ({ exec, useListen }) => {
  return (
    <div
      className={cn(
        "w-full h-full",
        "flex items-center justify-between",
        "bg-transparent",
        "select-none"
      )}
    >
      <div className={cn("flex items-center gap-4", "py-2")}>
        <Workspaces exec={exec} useListen={useListen} />
        <Spotify exec={exec} useListen={useListen} />
      </div>
      <Clock />
      <SysInfo exec={exec} useListen={useListen} />
    </div>
  );
};

export default Topbar;
