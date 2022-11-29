import {
  ActionIcon,
  Box,
  Button,
  Card,
  Container,
  createStyles,
  Group,
  keyframes,
  Loader,
  Modal,
  ScrollArea,
  Stack,
  StackProps,
  Table,
  Text,
  ThemeIcon
} from "@mantine/core"
import {
  IconAlertTriangle,
  IconAnalyze,
  IconChevronLeft,
  IconCircleCheck,
  IconFilter,
  IconHomeCheck,
  IconHomeExclamation,
  IconHomeSearch,
  IconRefreshAlert,
  IconSearch,
  IconSquare,
  IconTool
} from "@tabler/icons";
import {useTheme} from "@emotion/react";
import {AnimatePresence, motion} from "framer-motion";
import {getMainLayout} from "../../components/mainLayout";
import {useRecoilState, useRecoilValue, useSetRecoilState} from "recoil";
import {
  actionBatchState,
  applyErrorsState,
  scanCurrentState,
  scanPageState,
  scanStepState,
  selectedAddActionBatchState,
  selectedRemoveActionBatchState
} from "../../states";
import React, {useMemo, useState} from "react";
import {PathText} from "../../components/pathText";
import {SelectionTable} from "../../components/selectionTable";
import {ApplyErrors} from "../../bindings/ApplyErrors";
import {TipText} from "../../components/tipText";
import {useTableStyles} from "../../utils";

const circling = (radius: number) => keyframes({
  "from": {transform: `rotate(0deg) translateX(${radius}px) rotate(0deg)`},
  "to": {transform: `rotate(360deg) translateX(${radius}px) rotate(-360deg)`}
})

const useStyles = createStyles({
  circle: {
    animation: `${circling(10)} 2s linear infinite`,
  }
})

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

interface InProgressProps extends StackProps {
  onCancel: () => void
}

const InProgress = ({
                      onCancel,
                      ...props
                    }: InProgressProps) => {
  const {
    found,
    path
  } = useRecoilValue(scanCurrentState);
  const theme = useTheme();
  const {classes} = useStyles();
  const moreDimmed = theme.colorScheme === 'dark' ? theme.colors.dark[3] : theme.colors.gray[5];
  return (<Stack py={"xl"} align={"center"} justify={"center"} sx={{height: "100%"}} {...props}>
    <ThemeIcon size={128} radius={64} variant={"gradient"}>
      <IconSearch size={72} strokeWidth={1} className={classes.circle}/>
    </ThemeIcon>
    <Stack spacing={"xs"} align={"center"}>
      <Text size={"xl"}>Scanning system...</Text>
      <PathText size={"sm"} color={moreDimmed} align={"center"} lineClamp={1} keepFirst={4} keepLast={2} path={path}/>
      <Text size={"sm"} color={"dimmed"}>Found {found} file(s)</Text>
      <ActionIcon variant={"default"} radius={16} size={32} onClick={onCancel}>
        <IconSquare size={16} strokeWidth={1.5}/>
      </ActionIcon>
    </Stack>
  </Stack>)
}

interface ApplyingProps extends StackProps {
}

const Applying = (props: ApplyingProps) => {
  const {classes} = useStyles();
  return (<Stack py={"xl"} align={"center"} justify={"center"} sx={{height: "100%"}} {...props}>
    <ThemeIcon size={128} radius={64} variant={"gradient"}>
      <IconTool size={72} strokeWidth={1} className={classes.circle}/>
    </ThemeIcon>
    <Stack align={"center"} spacing={"lg"}>
      <Text size={"xl"}>Applying changes...</Text>
      <Group spacing={"xs"} align={"center"}>
        <Loader size={"xs"}/>
        <Text size={"xs"}>Setting file attributes</Text>
      </Group>
    </Stack>
  </Stack>)
}

interface DoneProps extends StackProps {
  onBack: () => void,
  selectedItems: number
}

const Done = ({
                onBack,
                selectedItems,
                ...props
              }: DoneProps) => {
  const theme = useTheme();
  const {classes, cx} = useTableStyles();
  const applyErrors = useRecoilValue(applyErrorsState);
  const [logOpened, setLogOpened] = useState(false);
  return (<Stack py={"xl"} sx={{height: "100%"}} {...props}>
    <Button pos={"absolute"} size={"xs"} sx={{boxShadow: "none"}} variant={"subtle"}
            leftIcon={<IconChevronLeft size={16} strokeWidth={1}/>}
            onClick={onBack}>
      Restart
    </Button>
    <Box sx={{flexGrow: 1}}/>
    <Group sx={{width: "100%"}} position={"center"}>
      <Group position={"center"} mr={"xl"}>
        <ThemeIcon size={128} radius={64}
                   variant={"gradient"}
                   gradient={(applyErrors === null) ? {from: "green", to: "lime"} : {from: "orange", to: "yellow"}}>
          {(applyErrors === null) ?
            <IconHomeCheck size={72} strokeWidth={1}/> :
            <IconHomeExclamation size={72} strokeWidth={1}/>}
        </ThemeIcon>
      </Group>
      <Stack spacing={"xs"}>
        <Text size={20}>Apply Complete</Text>
        <Group align={"end"} spacing={"xs"}>
          <Group align={"center"} spacing={"xs"}>
            {
              (applyErrors === null) ?
                <IconCircleCheck size={24} strokeWidth={1} color={theme.colors.lime[3]}/> :
                <IconAlertTriangle size={24} strokeWidth={1} color={theme.colors.orange[3]}/>
            }
            <Text size={28}>
              {selectedItems - Object.keys(applyErrors?.errors ?? {}).length} item(s)
            </Text>
          </Group>
          <Text size={"xs"} color={"dimmed"} pb={4}>applied</Text>
        </Group>
        <Modal centered size={"lg"} title={"Failed items"}
               opened={logOpened} onClose={() => setLogOpened(false)}
        >
          <ScrollArea sx={{height: "250px"}}>
            <Table highlightOnHover>
              <thead className={cx(classes.stickyHeader)}>
              <tr>
                <th>Path</th>
                <th>Reason</th>
              </tr>
              </thead>
              <tbody>
              {
                Object.entries(applyErrors?.errors ?? {}).map(([path, reason]) => (
                  <tr key={path}>
                    <td><PathText withinPortal keepFirst={4} keepLast={2} path={path} lineClamp={3}/></td>
                    <td><TipText withinPortal lineClamp={3}>{reason}</TipText></td>
                  </tr>
                ))
              }
              </tbody>
            </Table>
          </ScrollArea>
        </Modal>
        {
          (applyErrors === null) ?
            <Text size={"sm"}>Selected items has been excluded/included<br/> in TimeMachine backups.</Text> :
            <>
              <Text size={"sm"}>Some items failed to be excluded/included<br/> in TimeMachine backups.</Text>
              <Button size={"xs"} variant={"light"} color={"orange"} mr={"auto"}
                      onClick={() => setLogOpened(true)}>
                Show log
              </Button>
            </>
        }
      </Stack>
    </Group>
    <Box sx={{flexGrow: 1}}/>
  </Stack>)
}

interface OverviewProps extends StackProps {
  totalItems: number,
  selectedItems: number,
  onBack: () => void,
  onDetail: () => void,
  onApply: () => void
}

const Overview = ({
                    totalItems,
                    selectedItems,
                    onBack,
                    onDetail,
                    onApply,
                    ...props
                  }: OverviewProps) => {
  const theme = useTheme();
  return (<Stack py={"xl"} sx={{height: "100%"}} {...props}>
    <Button pos={"absolute"} size={"xs"} sx={{boxShadow: "none"}} variant={"subtle"}
            leftIcon={<IconChevronLeft size={16} strokeWidth={1}/>}
            onClick={onBack}>
      Restart
    </Button>
    <Box sx={{flexGrow: 1}}/>
    <Group sx={{width: "100%"}} position={"center"}>
      <Group position={"center"} mr={"xl"}>
        <ThemeIcon size={128} radius={64}
                   variant={"gradient"}
                   gradient={(selectedItems > 0) ?
                     {from: "orange", to: "yellow"} :
                     {from: "green", to: "lime"}}>
          {(selectedItems > 0) ?
            <IconHomeExclamation size={72} strokeWidth={1}/> :
            <IconHomeCheck size={72} strokeWidth={1}/>}
        </ThemeIcon>
      </Group>
      <Stack spacing={"xs"}>
        <Text size={20}>Scan Complete</Text>
        {(totalItems > 0) ? <>
          <Group align={"end"} spacing={"xs"}>
            <Text size={28} color={theme.colorScheme === "dark" ? theme.colors.blue[2] : theme.colors.blue[5]}>
              {selectedItems} items
            </Text>
            <Text size={"xs"} color={"dimmed"} pb={4}>selected</Text>
          </Group>
          <Group align={"center"}>
            <Button size={"xs"} variant={"light"} onClick={onDetail} sx={{boxShadow: "none"}}>View items</Button>
            <Text size={"xs"} color={"dimmed"}>{totalItems} items found</Text>
          </Group>
        </> : <Text size={"sm"}>
          Everything looks good!<br/>
          No files need to be excluded.
        </Text>}
      </Stack>
    </Group>
    {(selectedItems > 0) && <Stack align={"center"} spacing={"xs"} mt={"xl"}>
      <Button variant={"gradient"} leftIcon={<IconFilter/>} onClick={onApply}>Apply</Button>
      <Text size={"xs"} color={"dimmed"}>Exclude/include selected files from TimeMachine backups</Text>
    </Stack>}
    <Box sx={{flexGrow: 1}}/>
  </Stack>)
}

type DetailSelection = {
  addSelection: Array<string>,
  removeSelection: Array<string>,
  setAddSelection: React.Dispatch<React.SetStateAction<Array<string>>>
  setRemoveSelection: React.Dispatch<React.SetStateAction<Array<string>>>
};

type DetailData = {
  addData: Array<string>,
  removeData: Array<string>,
}

interface DetailProps extends StackProps {
  onBack: () => void,
  data: DetailData,
  selection: DetailSelection,
}

const Detail = ({
                  onBack,
                  data,
                  selection,
                  ...props
                }: DetailProps) => {
  const {addSelection, removeSelection, setAddSelection, setRemoveSelection} = selection;
  const {addData, removeData} = data;
  return (<Stack py={"xl"} sx={{
    height: "100%",
  }} {...props}>
    <Button size={"xs"} mr={"auto"} sx={{boxShadow: "none"}} variant={"subtle"}
            leftIcon={<IconChevronLeft size={16} strokeWidth={1}/>}
            onClick={onBack}>
      Back
    </Button>
    <Stack spacing={"xs"} sx={{height: "100%"}}>
      <Text size={"xs"} color={"dimmed"}>Files to be excluded</Text>
      <SelectionTable
        sx={{flexGrow: 1, flexBasis: 0}}
        data={addData} selection={addSelection} onChange={setAddSelection}
      />
      <Text size={"xs"} color={"dimmed"}>Files to be included</Text>
      <SelectionTable
        sx={{flexGrow: 1, flexBasis: 0}}
        data={removeData} selection={removeSelection} onChange={setRemoveSelection}
      />
    </Stack>
  </Stack>)
}

const Scan = () => {
  const scanStep = useRecoilValue(scanStepState);
  const [scanPage, setScanPage] = useRecoilState(scanPageState);

  const actionBatch = useRecoilValue(actionBatchState);
  const [addSelection, setAddSelection] = useRecoilState(selectedAddActionBatchState);
  const [removeSelection, setRemoveSelection] = useRecoilState(selectedRemoveActionBatchState);

  const setApplyErrors = useSetRecoilState(applyErrorsState);

  const totalItems = (actionBatch?.add.length ?? 0) + (actionBatch?.remove.length ?? 0);
  const selectedItems = addSelection.length + removeSelection.length;

  const detailData = useMemo(() => ({
    addData: actionBatch.add,
    removeData: actionBatch.remove,
  }), [actionBatch.add, actionBatch.remove]);
  const detailSelection = useMemo(() => ({
    addSelection,
    removeSelection,
    setAddSelection,
    setRemoveSelection,
  }), [addSelection, removeSelection, setAddSelection, setRemoveSelection]);

  const start_scan = async () => {
    const invoke = await import("@tauri-apps/api").then(tauri => tauri.invoke);
    await invoke("start_full_scan");
  }
  const stop_scan = async () => {
    const invoke = await import("@tauri-apps/api").then(tauri => tauri.invoke);
    await invoke("stop_full_scan");
  }
  const apply = async () => {
    const invoke = await import("@tauri-apps/api").then(tauri => tauri.invoke);
    setScanPage("applying");
    console.log({
      add: addSelection,
      remove: removeSelection,
    });
    try {
      await invoke("apply_action_batch", {
        batch: {
          add: addSelection,
          remove: removeSelection,
        }
      });
      setApplyErrors(null);
    } catch (_e: any) {
      const e = _e as ApplyErrors;
      setApplyErrors(e);
    }
    setScanPage("done");
  }
  const fadeAnimation = {
    initial: {opacity: 0},
    animate: {opacity: 1},
    exit: {opacity: 0},
  }
  const slideFadeAnimation = {
    initial: {x: 50, opacity: 0},
    animate: {x: 0, opacity: 1},
    exit: {x: 50, opacity: 0},
    transition: {ease: "easeOut"}
  };
  return <Container sx={{height: "100%"}}>
    <AnimatePresence mode={"popLayout"} initial={false}>
      {
        scanPage === "scan" ? (
            scanStep === "idle" ?
              <motion.div key={"welcome"} style={{height: "100%"}} {...fadeAnimation}>
                <Welcome onScan={() => start_scan()}/>
              </motion.div> :
              scanStep === "scanning" ?
                <motion.div key={"inProgress"} style={{height: "100%"}} {...fadeAnimation}>
                  <InProgress onCancel={() => stop_scan()}/>
                </motion.div> :
                <motion.div key={"overview"} style={{height: "100%"}} {...slideFadeAnimation}>
                  <Overview
                    totalItems={totalItems} selectedItems={selectedItems}
                    onBack={() => stop_scan()}
                    onDetail={() => setScanPage("detail")}
                    onApply={() => apply()}
                  />
                </motion.div>
          ) :
          scanPage === "detail" ?
            <motion.div key={"detail"} style={{height: "100%"}} {...slideFadeAnimation}>
              <Detail data={detailData} selection={detailSelection}
                      onBack={() => setScanPage("scan")}/>
            </motion.div> :
            scanPage === "applying" ?
              <motion.div key={"applying"} style={{height: "100%"}} {...fadeAnimation}>
                <Applying/>
              </motion.div> :
              <motion.div key={"done"} style={{height: "100%"}} {...fadeAnimation}>
                <Done onBack={() => {
                  const f = async () => {
                    await stop_scan();
                    setScanPage("scan");
                  }
                  f();
                }} selectedItems={selectedItems}/>
              </motion.div>
      }
    </AnimatePresence>
  </Container>
}

Scan.getLayout = getMainLayout;
export default Scan