import {getMainLayout} from "../../components/main-layout";
import {Badge, Button, Container, ScrollArea, Stack, Table, Text} from "@mantine/core";
import {PreDirectory} from "../../bindings/PreDirectory";
import {IconMinus, IconPlus} from "@tabler/icons";

const DEMO_DIRECTORIES: PreDirectory[] = [
    {path: "~/", rules: ["Development"]},
    {
        path: "~/Projects/iOS Project",
        rules: ["Python", "Gradle", "Gradle Kotlin Script", "Flutter (Dart)", "Maven", "Swift"]
    },
];

const DEMO_SKIPS: string[] = [
    "~/Library",
    "~/Pictures",
    "~/.vscode",
    "~/.npm",
    "~/Dropbox",
    "~/.dropbox",
    "~/.Trash",
    "~/.pnpm-store",
    "~/Downloads",
    "~/Desktop",
    "~/Documents",
];

const buttonStyles = {
    root: {paddingRight: 7},
    leftIcon: {marginRight: 0}
};

const Rules = () => (
    <Container sx={{height: "100%"}}>
        <Stack py={"xl"} sx={{height: "100%"}}>
            <Text>Directories to watch and scan</Text>
            <ScrollArea sx={{height: "40%"}}>
                <Table highlightOnHover>
                    <tbody>
                    {
                        DEMO_DIRECTORIES.map(({path, rules}) => (
                            <tr key={path}>
                                <td><Text>{path}</Text></td>
                                <td>{rules.map((rule) => (
                                    <Badge variant={"light"}><Text size={9}>{rule}</Text></Badge>
                                ))}</td>
                            </tr>
                        ))
                    }
                    </tbody>
                </Table>
            </ScrollArea>
            <Button.Group ml={"auto"}>
                <Button size={"xs"} compact leftIcon={<IconPlus size={12}/>} styles={buttonStyles} variant={"default"}/>
                <Button size={"xs"} compact leftIcon={<IconMinus size={12}/>} styles={buttonStyles} variant={"default"}/>
            </Button.Group>
            <Text>Skip the following paths</Text>
            <ScrollArea sx={{height: "40%"}}>
                <Table highlightOnHover>
                    <tbody>
                    {
                        DEMO_SKIPS.map((path) => (
                            <tr key={path}>
                                <td>{path}</td>
                            </tr>
                        ))
                    }
                    </tbody>
                </Table>
            </ScrollArea>
            <Button.Group ml={"auto"}>
                <Button size={"xs"} compact leftIcon={<IconPlus size={12}/>} styles={buttonStyles} variant={"default"}/>
                <Button size={"xs"} compact leftIcon={<IconMinus size={12}/>} styles={buttonStyles} variant={"default"}/>
            </Button.Group>
        </Stack>
    </Container>
)

Rules.getLayout = getMainLayout;

export default Rules;
