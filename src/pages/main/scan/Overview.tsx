'use client';
import {motion} from "framer-motion";
import {Box, Button, Group, Stack, Text, ThemeIcon, useMantineTheme} from "@mantine/core";
import {IconChevronLeft, IconFilter, IconHomeCheck, IconHomeExclamation} from "@tabler/icons";
import React from "react";
import {useRecoilValue, useSetRecoilState} from "recoil";
import {
  actionBatchState,
  applyErrorsState,
  scanPageState,
  selectedAddActionBatchState,
  selectedRemoveActionBatchState
} from "../../../states";
import {ApplyErrors} from "../../../bindings/ApplyErrors";
import {slideFadeAnimation} from "../../../transitions";
import {applyActionBatch, stopFullScan} from "../../../commands";

export const Overview = React.forwardRef(() => {
  const theme = useMantineTheme();

  const actionBatch = useRecoilValue(actionBatchState);
  const addSelection = useRecoilValue(selectedAddActionBatchState);
  const removeSelection = useRecoilValue(selectedRemoveActionBatchState);
  const setScanPage = useSetRecoilState(scanPageState);
  const setApplyErrors = useSetRecoilState(applyErrorsState);

  const totalItems = (actionBatch?.add.length ?? 0) + (actionBatch?.remove.length ?? 0);
  const selectedItems = addSelection.length + removeSelection.length;

  const apply = async () => {
    setScanPage("applying");
    try {
      await applyActionBatch({
        add: addSelection,
        remove: removeSelection,
      });
      setApplyErrors(null);
    } catch (_e: any) {
      const e = _e as ApplyErrors;
      setApplyErrors(e);
    }
    setScanPage("done");
  }

  return (
    <motion.div key={"overview"} style={{height: "100%"}} {...slideFadeAnimation}>
      <Stack py={"xl"} sx={{height: "100%"}}>
        <Button pos={"absolute"} size={"xs"} sx={{boxShadow: "none"}} variant={"subtle"}
                leftIcon={<IconChevronLeft size={16} strokeWidth={1}/>}
                onClick={stopFullScan}>
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
                <Button size={"xs"} variant={"light"} sx={{boxShadow: "none"}}
                        onClick={() => setScanPage("detail")}>
                  View items
                </Button>
                <Text size={"xs"} color={"dimmed"}>{totalItems} items found</Text>
              </Group>
            </> : <Text size={"sm"}>
              Everything looks good!<br/>
              No files need to be excluded.
            </Text>}
          </Stack>
        </Group>
        {(selectedItems > 0) && <Stack align={"center"} spacing={"xs"} mt={"xl"}>
          <Button variant={"gradient"} leftIcon={<IconFilter/>} onClick={apply}>Apply</Button>
          <Text size={"xs"} color={"dimmed"}>Exclude/include selected files from TimeMachine backups</Text>
        </Stack>}
        <Box sx={{flexGrow: 1}}/>
      </Stack>
    </motion.div>)
});
