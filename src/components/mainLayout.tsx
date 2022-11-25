import {Box, Button, Container, Group, Header, Navbar, NavLink, Text} from "@mantine/core";
import {Fragment, ReactElement} from "react";
import {IconAdjustments, IconChartBubble, IconFolders, IconHomeSearch, IconSettings, IconTemplate} from "@tabler/icons";
import {useRouter} from "next/router";
import Link from "next/link";
import {motion} from "framer-motion";
import {useElementSize, useViewportSize} from "@mantine/hooks";
import {useRecoilValue, useResetRecoilState, useSetRecoilState} from "recoil";
import {configChangedState, draftConfigState, finalConfigState} from "../states";

type NavLink = {
  kind: "link", title: string, icon: ReactElement, href: string
}

type NavTitle = {
  kind: "title", title: string
}

type NavItem = NavLink | NavTitle

const routes: NavItem[] = [{
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

const variants = {
  hidden: {opacity: 0},
  enter: {opacity: 1},
  exit: {opacity: 0},
}

export const MainLayout = ({children}: { children: ReactElement }) => {
  const router = useRouter();
  const {
    ref,
    height: headerHeight
  } = useElementSize();
  const {height: vh} = useViewportSize();
  const evDrag = async (ev: { preventDefault: () => void; }) => {
    const {appWindow} = await import("@tauri-apps/api/window");
    ev.preventDefault();
    await appWindow.startDragging();
  };
  const navbar = <Navbar width={{base: 250}} height={600} p="xs"
                         styles={{
                           root: {
                             background: "transparent",
                             boxShadow: "-3px 0px 3px -3px rgba(0,0,0,0.1) inset"
                           }
                         }}>
    <Box sx={{height: 40}} onMouseDown={evDrag}/>
    {routes.map((section, idx) => (section.kind === "link" ? <Fragment key={`nav-${idx}`}>
        <Navbar.Section>
          <Link key={`link-${section.title}`} href={section.href} passHref style={{textDecoration: "none"}}>
            <NavLink
              component={"div"}
              icon={section.icon}
              label={section.title}
              color={"dark"}
              active={router.pathname === section.href}
              sx={(theme) => ({borderRadius: theme.radius.sm})}
              styles={{
                label: {
                  cursor: "pointer"
                }
              }}
              py={3}
            />
          </Link>
        </Navbar.Section>
      </Fragment> : <Fragment key={`nav-${idx}`}>
        <Navbar.Section>
          <Text size={"sm"} weight={500} color={"dimmed"} px={"xs"} py={5}>{section.title}</Text>
        </Navbar.Section>
      </Fragment>))}
  </Navbar>;
  // TODO split header into independent component for better hook performance
  const changed = useRecoilValue(configChangedState);
  const resetDraft = useResetRecoilState(draftConfigState);
  const draftConfig = useRecoilValue(draftConfigState);
  const setFinalConfig = useSetRecoilState(finalConfigState);
  const header = <Header ref={ref} height={55} p="xs"
                         styles={(theme) => ({
                           root: {
                             background: theme.colorScheme === "dark" ? "#38343C" : "#F6F2F9",
                           }
                         })}
                         onMouseDown={evDrag}>
    <Group spacing={"xs"} p={5} sx={(theme) => ({
      color: theme.colorScheme === "dark" ? "#ffffff" : "inherit",
      alignItems: "flex-start"
    })}>
      <Box sx={{
        height: 20,
        width: 20
      }} pt={2}>
        <IconAdjustments size={20} strokeWidth={1.5}/>
      </Box>
      <Text size={"md"}>Preference</Text>
      {changed && <>
        <Box sx={{flexGrow: 1}}/>
        <Button variant={"subtle"} compact sx={{boxShadow: "none"}} onClick={() => resetDraft()}>Reset</Button>
        <Button compact onClick={() => setFinalConfig(draftConfig)}>Save</Button>
      </>}
    </Group>
  </Header>;
  return (<Box sx={{
      display: "flex",
      flexDirection: "row"
    }} m={"auto"}>
      {navbar}
      <Box sx={{
        width: "100%",
        display: "flex",
        flexDirection: "column",
        overflowX: "hidden"
      }}>
        {header}
        <Container
          sx={(theme) => ({
            backgroundColor: theme.colorScheme === 'dark' ? theme.colors.dark[8] : theme.colors.gray[0],
            height: vh - headerHeight - theme.spacing.xs * 2 - 1,
            width: "100%"
          })}>
          <motion.div
            key={router.asPath}
            variants={variants}
            initial={"hidden"}
            animate={"enter"}
            exit={"exit"}
            style={{height: "100%"}}
          >
            {children}
          </motion.div>
        </Container>
      </Box>
    </Box>)
};

export const getMainLayout = (page: ReactElement) => <MainLayout>{page}</MainLayout>;