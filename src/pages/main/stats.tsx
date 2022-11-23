import {getMainLayout} from "../../components/mainLayout";
import {Box, Card, Container, Group, Stack, ThemeIcon, Text, Title} from "@mantine/core";
import {IconCheck, IconDots, IconFileMinus, IconFilePlus, IconRadar} from "@tabler/icons";
import useSWR from 'swr'
import {Metrics} from "../../bindings/Metrics";
import ReactTimeago from "react-timeago";

const fetcher = async (key: string) => {
    if (typeof window === "undefined") {
        return null
    }
    const invoke = await import("@tauri-apps/api").then(tauri => tauri.invoke);
    return await invoke<any>(key);
}

const Stats = () => {
    const {data} = useSWR<Metrics>("metrics", fetcher);
    return (
        <Container>
            <Stack py={"xl"}>
                <Box pl={"xl"} pb={"xl"}>
                    <Title order={2}>Looks good!</Title>
                    <Text size={"sm"}>TimeMachine Exclude is running.</Text>
                </Box>
                <Card radius={"lg"} withBorder>
                    <Group>
                        <ThemeIcon radius={"md"} size={50} variant={"gradient"}
                                   gradient={{from: "cyan", to: "indigo", deg: 105}}>
                            <IconFilePlus size={30} strokeWidth={1.5}/>
                        </ThemeIcon>
                        <Box>
                            <Group spacing={"xs"}>
                                <ThemeIcon size={16} variant={"outline"} radius={"xl"} color={"green"}>
                                    <IconCheck size={12} strokeWidth={3}/>
                                </ThemeIcon>
                                <Text size={"xl"}>{data?.files_excluded} Files</Text>
                            </Group>
                            <Text size={"sm"} color={"dimmed"}>have been excluded from TimeMachine backups</Text>
                        </Box>
                    </Group>
                </Card>
                <Card radius={"lg"} withBorder>
                    <Group>
                        <ThemeIcon radius={"md"} size={50} variant={"gradient"}
                                   gradient={{from: "lime", to: "teal", deg: 105}}>
                            <IconFileMinus size={30} strokeWidth={1.5}/>
                        </ThemeIcon>
                        <Box>
                            <Group spacing={"xs"}>
                                <ThemeIcon size={16} variant={"outline"} radius={"xl"} color={"green"}>
                                    <IconCheck size={12} strokeWidth={3}/>
                                </ThemeIcon>
                                <Text size={"xl"}>{data?.files_included} Files</Text>
                            </Group>
                            <Text size={"sm"} color={"dimmed"}>have been
                                re-included into TimeMachine backups</Text>
                        </Box>
                    </Group>
                </Card>
                <Card radius={"lg"} withBorder>
                    <Group>
                        <ThemeIcon radius={"md"} size={50} variant={"gradient"}
                                   gradient={{from: "yellow", to: "pink", deg: 105}}>
                            <IconRadar size={30} strokeWidth={1.5}/>
                        </ThemeIcon>
                        <Box>
                            <Group spacing={"xs"}>
                                <ThemeIcon size={16} variant={"outline"} radius={"xl"} color={"orange"}>
                                    <IconDots size={12} strokeWidth={3}/>
                                </ThemeIcon>
                                <Text
                                    size={"xl"}>{data?.last_excluded ? data.last_excluded : "N/A"}</Text>
                            </Group>
                            <Text size={"sm"} color={"dimmed"}>{
                                (data && data.last_excluded_time !== 0) ?
                                    <p>
                                        was excluded <ReactTimeago date={data.last_excluded_time * 1000}/>
                                    </p>
                                    : "no files have been excluded yet"
                            }</Text>
                        </Box>
                    </Group>
                </Card>
            </Stack>
        </Container>
    )
};

Stats.getLayout = getMainLayout;

export default Stats;