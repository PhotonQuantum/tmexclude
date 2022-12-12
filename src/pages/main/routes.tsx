import {ReactElement} from "react";
import {IconChartBubble, IconFolders, IconHomeSearch, IconSettings, IconTemplate} from "@tabler/icons";
import {TFunction} from "i18next";

export type NavLink = {
  kind: "link", title: string, icon: ReactElement, href: string
}

export type NavTitle = {
  kind: "title", title: string
}

export type NavItem = NavLink | NavTitle

export const routes: (t: TFunction) => Array<NavItem> = (t: TFunction) => [{
  kind: "title",
  title: t('overview')
}, {
  kind: "link",
  title: t('statistics'),
  icon: <IconChartBubble size={16} strokeWidth={1.5}/>,
  href: "/main/stats"
}, {
  kind: "title",
  title: t('settings')
}, {
  kind: "link",
  title: t('general'),
  icon: <IconSettings size={16} strokeWidth={1.5}/>,
  href: "/main/general"
}, {
  kind: "link",
  title: t('rules'),
  icon: <IconTemplate size={16} strokeWidth={1.5}/>,
  href: "/main/rules"
}, {
  kind: "link",
  title: t('directories'),
  icon: <IconFolders size={16} strokeWidth={1.5}/>,
  href: "/main/directories"
}, {
  kind: "title",
  title: t('actions')
}, {
  kind: "link",
  title: t('scan'),
  icon: <IconHomeSearch size={16} strokeWidth={1.5}/>,
  href: "/main/scan"
},];
