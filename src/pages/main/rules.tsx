import {getMainLayout} from "../../components/main-layout";
import {
    Accordion,
    ActionIcon,
    Box,
    Button,
    Container,
    Group,
    Menu,
    MultiSelect,
    ScrollArea,
    SegmentedControl,
    Stack,
    Text
} from "@mantine/core";
import {PreRule} from "../../bindings/PreRule";
import {IconDots, IconPencil, IconPlus, IconTemplate, IconTrash} from "@tabler/icons";

const DEMO_RULES: Record<string, PreRule> = {
    "Development": ["Swift", "Gradle", "Gradle Kotlin Script", "Flutter (Dart)", "Python", "Maven"],
    "Swift": {
        "excludes": [".build"],
        "if-exists": ["Package.swift"]
    },
    "Gradle": {
        "excludes": ["build"],
        "if-exists": ["build.gradle"]
    },
    "Gradle Kotlin Script": {
        "excludes": ["build"],
        "if-exists": ["build.gradle.kts"]
    },
    "Flutter (Dart)": {
        "excludes": ["build"],
        "if-exists": ["pubspec.yaml"]
    },
    "Python": {
        "excludes": ["build"],
        "if-exists": ["setup.py"]
    },
    "Maven": {
        "excludes": ["target"],
        "if-exists": ["pom.xml"]
    }
}

const Rules = () => (
    <Container sx={{height: "100%"}}>
        <Stack py={"xl"} sx={{height: "100%"}}>
            <Group sx={{flexWrap: "nowrap"}}>
                <Box><IconTemplate size={32} strokeWidth={1}/></Box>
                <Box sx={{flexGrow: 1}}>
                    <Text>Exclude paths that match these patterns</Text>
                    <Text size={"sm"} color={"dimmed"}>
                        Patterns must be applied by at least one directory to take effect.
                    </Text>
                </Box>
                <Button variant={"default"} size={"xs"} leftIcon={<IconPlus size={16}/>}>
                    Add Rule
                </Button>
            </Group>
            <Box/>
            <ScrollArea sx={{height: "100%"}}>
                <Accordion
                    variant={"filled"}
                    radius={"xs"}
                    chevronPosition={"left"}
                    styles={(theme) => ({
                        item: {
                            borderBottomStyle: "solid",
                            borderBottomWidth: "1px",
                            borderBottomColor: theme.colorScheme === 'dark' ? theme.colors.dark[4] : theme.colors.gray[2],
                        },
                        control: {
                            padding: theme.spacing.sm
                        }
                    })}
                >
                    {
                        Object.entries(DEMO_RULES).map(([key, value]) => (
                            <Accordion.Item key={key} value={key}>
                                <Group sx={{flexWrap: "nowrap"}} mr={"sm"} spacing={0}>
                                    <Accordion.Control>{key}</Accordion.Control>
                                    <Menu>
                                        <Menu.Target>
                                            <ActionIcon size={"lg"}><IconDots size={16}/></ActionIcon>
                                        </Menu.Target>
                                        <Menu.Dropdown>
                                            <Menu.Item icon={<IconPencil size={14}/>}>Rename</Menu.Item>
                                            <Menu.Item color={"red"} icon={<IconTrash size={14}/>}>Delete</Menu.Item>
                                        </Menu.Dropdown>
                                    </Menu>
                                </Group>
                                <Accordion.Panel>
                                    <Stack spacing={"xs"}>
                                        <SegmentedControl
                                            size={"xs"}
                                            data={[{label: "Merge Rule", value: "merge"}, {
                                                label: "Concrete Rule",
                                                value: "concrete"
                                            }]}
                                            value={Array.isArray(value) ? "merge" : "concrete"}
                                        />
                                        {
                                            Array.isArray(value) ? (
                                                <>
                                                    <MultiSelect
                                                        searchable
                                                        data={Object.keys(DEMO_RULES).filter((k) => k !== key)}
                                                        defaultValue={value}
                                                        placeholder={"Pick all sub-rules to merge"}
                                                    />
                                                </>
                                            ) : (
                                                <>
                                                    <Text size="sm">Paths to exclude</Text>
                                                    <MultiSelect searchable creatable data={value.excludes.map((v) => ({
                                                        value: v,
                                                        label: v
                                                    }))} defaultValue={value.excludes}/>
                                                    <Text size="sm">
                                                        ... only if any of these paths exists in the same directory
                                                    </Text>
                                                    <MultiSelect searchable creatable
                                                                 data={value["if-exists"].map((v) => ({
                                                                     value: v,
                                                                     label: v
                                                                 }))}
                                                                 defaultValue={value["if-exists"]}
                                                    />
                                                </>
                                            )
                                        }
                                    </Stack>
                                </Accordion.Panel>
                            </Accordion.Item>
                        ))
                    }
                </Accordion>
            </ScrollArea>
        </Stack>
    </Container>
)

Rules.getLayout = getMainLayout;

export default Rules;
