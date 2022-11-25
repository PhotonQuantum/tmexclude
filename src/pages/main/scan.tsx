import {
  ActionIcon, Box, Button, Card, Container, createStyles, Group, keyframes, Stack, StackProps, Text, ThemeIcon
} from "@mantine/core"
import {
  IconAnalyze,
  IconChevronLeft,
  IconFilter,
  IconHomeCheck,
  IconHomeExclamation,
  IconHomeSearch,
  IconRefreshAlert,
  IconSearch,
  IconSquare
} from "@tabler/icons";
import {useTheme} from "@emotion/react";
import {useState} from "react";
import {AnimatePresence, motion} from "framer-motion";
import {getMainLayout} from "../../components/mainLayout";

interface WelcomeProps extends StackProps {
  onScan: () => void,
}

const Welcome = ({
                   onScan,
                   ...props
                 }: WelcomeProps) => {
  const theme = useTheme();
  return (<Stack py={"xl"} align={"center"} justify={"center"} sx={{height: "100%"}} {...props}>
    <ActionIcon size={128} radius={64} variant={"gradient"} onClick={onScan} sx={{boxShadow: theme.shadows.xl}}>
      <IconHomeSearch size={72} strokeWidth={1}/>
    </ActionIcon>
    <Text size={"xl"}>Run a manual scan</Text>
    <Card>
      <Stack>
        <Group>
          <IconAnalyze size={24} strokeWidth={1.5}/>
          <Text size={"sm"}>Run an initial full scan after setup.</Text>
        </Group>
        <Group>
          <IconRefreshAlert size={24} strokeWidth={1.5}/>
          <Text size={"sm"}>Re-sync file changes if incremental scans fail.</Text>
        </Group>
      </Stack>
    </Card>
  </Stack>)
}

const circling = (radius: number) => keyframes({
  "from": {transform: `rotate(0deg) translateX(${radius}px) rotate(0deg)`},
  "to": {transform: `rotate(360deg) translateX(${radius}px) rotate(-360deg)`}
})

const useStyles = createStyles(() => ({
  circle: {
    animation: `${circling(10)} 2s linear infinite`,
  }
}))

interface InProgressProps extends StackProps {
  onCancel: () => void
}

const InProgress = ({
                      onCancel,
                      ...props
                    }: InProgressProps) => {
  const theme = useTheme();
  const {classes} = useStyles();
  const moreDimmed = theme.colorScheme === 'dark' ? theme.colors.dark[3] : theme.colors.gray[5];
  return (<Stack py={"xl"} align={"center"} justify={"center"} sx={{height: "100%"}} {...props}>
    <ThemeIcon size={128} radius={64} variant={"gradient"}>
      <IconSearch size={72} strokeWidth={1} className={classes.circle}/>
    </ThemeIcon>
    <Stack spacing={"xs"} align={"center"}>
      <Text size={"xl"}>Scanning system...</Text>
      <Text size={"sm"} color={moreDimmed}>/User/lightquantum/blabla</Text>
      <Text size={"sm"} color={"dimmed"}>Found 1 file(s)</Text>
      <ActionIcon variant={"default"} radius={16} size={32} onClick={onCancel}>
        <IconSquare size={16} strokeWidth={1.5}/>
      </ActionIcon>
    </Stack>
  </Stack>)
}

interface ReviewProps extends StackProps {
  files: Record<string, boolean>
  onDetail: () => void
  onBack: () => void
}

const DEMO_FILES: Record<string, boolean> = {
  "/Users/lightquantum/test.txt": true,
  "/Users/lightquantum/test2.txt": true,
  "/Users/lightquantum/test3.txt": true,
  "/Users/lightquantum/test4.txt": true,
  "/Users/lightquantum/test5.txt": false
}

const DEMO_FILES_EMPTY: Record<string, boolean> = {}

const Report = ({
                  files,
                  onDetail,
                  onBack,
                  ...props
                }: ReviewProps) => {
  const theme = useTheme();
  const hasFiles = Object.keys(files).length > 0;
  return (<Stack py={"xl"} sx={{
    height: "100%",
  }} {...props}>
    <Button pos={"absolute"} size={"xs"} sx={{boxShadow: "none"}} variant={"subtle"}
            leftIcon={<IconChevronLeft size={16} strokeWidth={1}/>}
            onClick={onBack}>
      Restart
    </Button>
    <Box sx={{flexGrow: 1}}/>
    <Group sx={{
      width: "100%"
    }} position={"center"}>
      <Group position={"center"} mr={"xl"}>
        <ThemeIcon size={128} radius={64} variant={"gradient"} gradient={hasFiles ? {
          from: "orange",
          to: "yellow"
        } : {
          from: "green",
          to: "lime"
        }}>
          {hasFiles ? <IconHomeExclamation size={72} strokeWidth={1}/> : <IconHomeCheck size={72} strokeWidth={1}/>}
        </ThemeIcon>
      </Group>
      <Stack spacing={"xs"}>
        <Text size={20}>Scan Complete</Text>
        {hasFiles ? <>
          <Group align={"end"} spacing={"xs"}>
            <Text size={28} color={theme.colorScheme === "dark" ? theme.colors.blue[2] : theme.colors.blue[5]}>114514
              items</Text>
            <Text size={"xs"} color={"dimmed"} pb={4}>selected</Text>
          </Group>
          <Group align={"center"}>
            <Button size={"xs"} variant={"light"} onClick={onDetail} sx={{boxShadow: "none"}}>View items</Button>
            <Text size={"xs"} color={"dimmed"}>114514 items found</Text>
          </Group>
        </> : <Text size={"sm"}>
          Everything looks good!<br/>
          No files need to be excluded.
        </Text>}
      </Stack>
    </Group>
    {hasFiles && <Stack align={"center"} spacing={"xs"} mt={"xl"}>
      <Button variant={"gradient"} leftIcon={<IconFilter/>}>Exclude</Button>
      <Text size={"xs"} color={"dimmed"}>Exclude selected files from TimeMachine backups</Text>
    </Stack>}
    <Box sx={{flexGrow: 1}}/>
  </Stack>)
}

const Scan = () => {
  const [step, setStep] = useState(0);
  return (<Container sx={{height: "100%"}}>
    <AnimatePresence mode={"popLayout"} initial={false}>
      {(step === 0) ? <motion.div key={"welcome"} style={{height: "100%"}}
                                  initial={{opacity: 0}} animate={{opacity: 1}} exit={{opacity: 0}}>
        <Welcome onScan={() => {
          setStep(1);
          setTimeout(() => {
            setStep(2);
          }, 3000)
        }}/>
      </motion.div> : (step === 1) ? <motion.div key={"inProgress"} style={{height: "100%"}}
                                                 initial={{opacity: 0}} animate={{opacity: 1}} exit={{opacity: 0}}>
        <InProgress onCancel={() => setStep(0)}/>
      </motion.div> : <motion.div key={"review"} style={{height: "100%"}}
                                  initial={{
                                    x: 50,
                                    opacity: 0
                                  }}
                                  animate={{
                                    x: 0,
                                    opacity: 1
                                  }}
                                  exit={{
                                    x: 50,
                                    opacity: 0
                                  }}
                                  transition={{ease: "easeOut"}}
      >
        <Report files={DEMO_FILES_EMPTY} onDetail={() => {
          setStep(0);
        }}
                onBack={() => {
                  setStep(0);
                }}
        />
      </motion.div>}
    </AnimatePresence>
  </Container>)
}

Scan.getLayout = getMainLayout;
export default Scan