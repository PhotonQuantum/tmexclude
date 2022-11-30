'use client';
import {ReactElement} from "react";
import {IconChartBubble, IconFolders, IconHomeSearch, IconSettings, IconTemplate} from "@tabler/icons";

export type NavLink = {
  kind: "link", title: string, icon: ReactElement, href: string
}

export type NavTitle = {
  kind: "title", title: string
}

export type NavItem = NavLink | NavTitle

export const routes: NavItem[] = [{
  kind: "title",
  title: "Overview"
}, {
  kind: "link",
  title: "Statistics",
  icon: <IconChartBubble size={16} strokeWidth={1.5}/>,
  href: "/main/stats"
}, {
  kind: "title",
  title: "Settings"
}, {
  kind: "link",
  title: "General",
  icon: <IconSettings size={16} strokeWidth={1.5}/>,
  href: "/main/general"
}, {
  kind: "link",
  title: "Rules",
  icon: <IconTemplate size={16} strokeWidth={1.5}/>,
  href: "/main/rules"
}, {
  kind: "link",
  title: "Directories",
  icon: <IconFolders size={16} strokeWidth={1.5}/>,
  href: "/main/directories"
}, {
  kind: "title",
  title: "Actions"
}, {
  kind: "link",
  title: "Scan",
  icon: <IconHomeSearch size={16} strokeWidth={1.5}/>,
  href: "/main/scan"
},];
