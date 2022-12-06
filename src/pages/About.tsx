import useSWR from "swr";
import {evDrag, swrFetcher} from "../utils";
import {BuildMeta} from "../bindings/BuildMeta";
import {Box, Container, Divider, Group, Image, Stack, Text, Title} from "@mantine/core";
// @ts-ignore
import icon from "../assets/harddisk.png";

export const About = () => {
  const {data} = useSWR<BuildMeta>("build_meta", swrFetcher);
  const date = new Date(data?.timestamp ?? 0);
  const dateString = date.toLocaleDateString();
  const buildYear = date.getUTCFullYear();

  const openAck = async () => {
    const WebviewWindow = await import("@tauri-apps/api/window").then(window => window.WebviewWindow);
    const target = WebviewWindow.getByLabel("ack")!;
    await target.show();
  }

  const openLicense = async () => {
    const WebviewWindow = await import("@tauri-apps/api/window").then(window => window.WebviewWindow);
    const target = WebviewWindow.getByLabel("license")!;
    await target.show();
  }

  const A = (props: { href: string, children: React.ReactNode }) => (
    <Text inline sx={{cursor: "pointer"}} c={"blue"} component={"a"} target={"_blank"} {...props}/>
  )
  return (
    <>
      <Box pt={24} onMouseDown={evDrag}/>
      <Container pl={24} pt={12}>
        <Group spacing={"xl"} align={"flex-start"}>
          <Image src={icon} width={64}/>
          <Stack spacing={"xs"}>
            <Stack spacing={0}>
              <Title order={4}>TimeMachine Exclude</Title>
              <Text size={"xs"}>Build {data?.version ?? "unknown"}, built on {dateString}</Text>
            </Stack>
            <Divider/>
            <Text size={"xs"}>
              Github: <A href={"https://github.com/PhotonQuantum/tmexclude"}>
              https://github.com/PhotonQuantum/tmexclude
            </A>
            </Text>
            <Divider/>
            <Title order={6}>Contact Me</Title>
            <Text size={"xs"}>
              Github - <A href={"https://github.com/PhotonQuantum"}>@PhotonQuantum</A><br/>
              Twitter - <A href={"https://twitter.com/LightQuantumhah"}>@LightQuantumhah</A>
            </Text>
            <Divider/>
            <Text size={"xs"}>
              Licensed under <Text span inline sx={{cursor: "pointer"}} c={"blue"}
                                   onClick={openLicense}>MIT License</Text><br/>
              Powered by <Text span inline sx={{cursor: "pointer"}} c={"blue"}
                               onClick={openAck}>open-source software</Text><br/>
              Copyright © {buildYear} <A href={"https://github.com/PhotonQuantum"}>LightQuantum</A>
            </Text>
          </Stack>
        </Group>
      </Container>
    </>
  )
}