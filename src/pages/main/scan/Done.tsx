import {useTableStyles} from "../../../utils";
import {useRecoilValue, useSetRecoilState} from "recoil";
import {
  applyErrorsState,
  scanPageState,
  selectedAddActionBatchState,
  selectedRemoveActionBatchState
} from "../../../states";
import React, {useState} from "react";
import {Box, Button, Group, Modal, ScrollArea, Stack, Table, Text, ThemeIcon, useMantineTheme} from "@mantine/core";
import {IconAlertTriangle, IconChevronLeft, IconCircleCheck, IconHomeCheck, IconHomeExclamation} from "@tabler/icons";
import {motion} from "framer-motion";
import {fadeAnimation} from "../../../transitions";
import {TipText} from "../../../components/TipText";
import {PathText} from "../../../components/PathText";
import {stopFullScan} from "../../../commands";

export const Done = React.forwardRef(() => {
  const theme = useMantineTheme();
  const {classes, cx} = useTableStyles();

  const applyErrors = useRecoilValue(applyErrorsState);
  const addSelection = useRecoilValue(selectedAddActionBatchState);
  const removeSelection = useRecoilValue(selectedRemoveActionBatchState);
  const setScanPage = useSetRecoilState(scanPageState);

  const selectedItems = addSelection.length + removeSelection.length;

  const [logOpened, setLogOpened] = useState(false);

  const onBack = async () => {
    await stopFullScan();
    setScanPage("scan");
  };

  return (
    <motion.div key={"done"} style={{height: "100%"}} {...fadeAnimation}>
      <Stack py={"xl"} sx={{height: "100%"}}>
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
      </Stack>
    </motion.div>)
});
