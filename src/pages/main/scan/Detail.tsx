import {Button, Group, Stack, Text, Tooltip} from "@mantine/core";
import {IconAlertTriangle, IconChevronLeft} from "@tabler/icons";
import React, {useState} from "react";
import {useRecoilState, useRecoilValue, useSetRecoilState} from "recoil";
import {
  actionBatchState,
  scanPageState,
  selectedAddActionBatchState,
  selectedRemoveActionBatchState
} from "../../../states";
import {motion} from "framer-motion";
import {slideFadeAnimation} from "../../../transitions";
import {SelectionTable} from "../../../components/SelectionTable";
import {useTranslation} from "react-i18next";

export const Detail = React.forwardRef(() => {
  const {t} = useTranslation();

  const {add: addData, remove: removeData} = useRecoilValue(actionBatchState);
  const [addSelection, setAddSelection] = useRecoilState(selectedAddActionBatchState);
  const [removeSelection, setRemoveSelection] = useRecoilState(selectedRemoveActionBatchState);
  const setScanPage = useSetRecoilState(scanPageState);

  const [addTrunc, setAddTrunc] = useState<number | null>(null);
  const [removeTrunc, setRemoveTrunc] = useState<number | null>(null);

  const truncatedNote = (count: number) => (
    <Tooltip label={t("showing_n_rows", {count})}>
      <Group spacing={"xs"}>
        <Text size={"xs"} color={"orange"}>{t('truncated')}</Text>
        <IconAlertTriangle size={16} strokeWidth={1} color={"orange"}/>
      </Group>
    </Tooltip>);

  return (
    <motion.div key={"detail"} style={{height: "100%"}} {...slideFadeAnimation}>
      <Stack py={"xl"} sx={{
        height: "100%",
      }}>
        <Button size={"xs"} mr={"auto"} sx={{boxShadow: "none"}} variant={"subtle"}
                leftIcon={<IconChevronLeft size={16} strokeWidth={1}/>}
                onClick={() => setScanPage("scan")}>
          {t('back')}
        </Button>
        <Stack spacing={"xs"} sx={{height: "100%"}}>
          <Group position={"apart"}>
            <Text size={"xs"} color={"dimmed"}>{t('files_to_be_excluded')}</Text>
            {addTrunc !== null && truncatedNote(addTrunc)}
          </Group>
          <SelectionTable
            sx={{flexGrow: 1, flexBasis: 0}}
            data={addData} selection={addSelection}
            limit={100} onChange={setAddSelection} onTruncated={setAddTrunc}
          />
          <Group position={"apart"}>
            <Text size={"xs"} color={"dimmed"}>{t('files_to_be_included')}</Text>
            {removeTrunc !== null && truncatedNote(removeTrunc)}
          </Group>
          <SelectionTable
            sx={{flexGrow: 1, flexBasis: 0}}
            data={removeData} selection={removeSelection}
            limit={100} onChange={setRemoveSelection} onTruncated={setRemoveTrunc}
          />
        </Stack>
      </Stack>
    </motion.div>)
});