import {AppShell, Box, Container, Group, Header, Image, Navbar, NavLink, useMantineTheme} from "@mantine/core";
import logo from "../assets/harddisk.png";
import {Fragment, ReactElement} from "react";
import {IconChartBubble, IconFolders, IconHomeSearch, IconSettings, IconTemplate} from "@tabler/icons";
import {useRouter} from "next/router";
import Link from "next/link";
import {motion} from "framer-motion";
import {Text} from "./text";
import {useElementSize, useViewportSize} from "@mantine/hooks";

const routes = [
    [
        {title: "Statistics", icon: <IconChartBubble size={16} strokeWidth={1.5}/>, href: "/main/stats"},
    ],
    [
        {title: "General", icon: <IconSettings size={16} strokeWidth={1.5}/>, href: "/main/general"},
        {title: "Rules", icon: <IconTemplate size={16} strokeWidth={1.5}/>, href: "/main/rules"},
        {title: "Directories", icon: <IconFolders size={16} strokeWidth={1.5}/>, href: "/main/directories"},
    ],
    [
        {title: "Scan", icon: <IconHomeSearch size={16} strokeWidth={1.5}/>, href: "/main/scan"},
    ]
];

const variants = {
    hidden: {opacity: 0},
    enter: {opacity: 1},
    exit: {opacity: 0},
}

export const MainLayout = ({children}: { children: ReactElement }) => {
    const router = useRouter();
    const theme = useMantineTheme();
    const {ref} = useElementSize();
    const {height} = useViewportSize();
    return (
        <AppShell
            navbar={
                <Navbar width={{base: 200}} height={600} p="xs"
                        styles={() => ({root: {background: "transparent"}})}>
                    {routes.map((section, idx) => (
                        <Fragment key={`nav-${idx}`}>
                            {idx > 0 &&
                                <Navbar.Section>
                                    <Box sx={{
                                        borderTop: `1px solid ${
                                            theme.colorScheme === 'dark' ? theme.colors.dark[4] : theme.colors.gray[5]
                                        }`,
                                    }} my={"xs"} key={`nav-box-${idx}-separator`}/>
                                </Navbar.Section>
                            }
                            <Navbar.Section>
                                {section.map((route, idx) => (
                                    <Link key={`link-${route.title}`} href={route.href} passHref>
                                        <NavLink
                                            component={"a"}
                                            icon={route.icon}
                                            label={route.title}
                                            color={"dark"}
                                            active={router.pathname === route.href}
                                            sx={{borderRadius: theme.radius.md}}
                                        />
                                    </Link>))}
                            </Navbar.Section>
                        </Fragment>
                    ))}
                </Navbar>
            }
            header={
                <Header ref={ref} height={45} p="xs"
                        styles={() => ({root: {background: "transparent"}})}
                        onMouseDown={async ev => {
                            const {appWindow} = await import("@tauri-apps/api/window");
                            ev.preventDefault();
                            await appWindow.startDragging();
                        }}>
                    <Group spacing={"sm"}>
                        <Image src={logo.src} alt="logo" height={25} width={25}/>
                        <Text size={"md"}>TimeMachine Exclude</Text>
                    </Group>
                </Header>
            }
            styles={(theme) => ({
                // main: {backgroundColor: theme.colorScheme === 'dark' ? theme.colors.dark[8] : theme.colors.gray[0]},
                main: {backgroundColor: "transparent", height: height},
            })}
        >
            <Container mx={-theme.spacing.md} my={-theme.spacing.md}
                       sx={{
                           backgroundColor: theme.colorScheme === 'dark' ? theme.colors.dark[8] : theme.colors.gray[0],
                           height: `calc(100% + ${theme.spacing.md * 2}px)`
                       }}>
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
        </AppShell>
    )
};

export const getMainLayout = (page: ReactElement) => <MainLayout>{page}</MainLayout>;