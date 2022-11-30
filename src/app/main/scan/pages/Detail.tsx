'use client';
import {Button, Stack, Text} from "@mantine/core";
import {IconChevronLeft} from "@tabler/icons";
import React from "react";
import {useRecoilState, useRecoilValue, useSetRecoilState} from "recoil";
import {
  actionBatchState,
  scanPageState,
  selectedAddActionBatchState,
  selectedRemoveActionBatchState
} from "../../../states";
import {motion} from "framer-motion";
import {slideFadeAnimation} from "../transitions";
import {SelectionTable} from "../SelectionTable";

export const Detail = React.forwardRef(() => {
  const {add: addData, remove: removeData} = useRecoilValue(actionBatchState);
  const [addSelection, setAddSelection] = useRecoilState(selectedAddActionBatchState);
  const [removeSelection, setRemoveSelection] = useRecoilState(selectedRemoveActionBatchState);
  const setScanPage = useSetRecoilState(scanPageState);

  return (
    <motion.div key={"detail"} style={{height: "100%"}} {...slideFadeAnimation}>
      <Stack py={"xl"} sx={{
        height: "100%",
      }}>
        <Button size={"xs"} mr={"auto"} sx={{boxShadow: "none"}} variant={"subtle"}
                leftIcon={<IconChevronLeft size={16} strokeWidth={1}/>}
                onClick={() => setScanPage("scan")}>
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
      </Stack>
    </motion.div>)
});