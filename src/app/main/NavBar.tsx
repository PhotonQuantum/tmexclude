'use client';
import {Box, Navbar, NavLink, Text} from "@mantine/core";
import {Fragment} from "react";
import Link from "next/link";
import {usePathname} from "next/navigation";
import {routes} from "./routes";

export const NavBar = () => {
  const pathname = usePathname();
  const evDrag = async (ev: { preventDefault: () => void; }) => {
    const {appWindow} = await import("@tauri-apps/api/window");
    ev.preventDefault();
    await appWindow.startDragging();
  };

  return (
    <Navbar width={{base: 250}} height={600} p="xs"
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
              active={pathname === section.href}
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
    </Navbar>);
}